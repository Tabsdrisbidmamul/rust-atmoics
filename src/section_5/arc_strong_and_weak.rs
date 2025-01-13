use std::{
    cell::UnsafeCell,
    mem::ManuallyDrop,
    ops::Deref,
    ptr::NonNull,
    sync::atomic::{fence, AtomicUsize, Ordering},
};

#[derive(Debug)]
struct ArcData<T> {
    /// Number of `Arc`s.
    data_ref_count: AtomicUsize,
    /// Number of `Weak`s, plus one if there are any `Arc`s.
    alloc_ref_count: AtomicUsize,
    /// The data. Dropped if there are only weak pointers left.
    data: UnsafeCell<ManuallyDrop<T>>,
}

pub struct Arc<T> {
    ptr: NonNull<ArcData<T>>,
}

unsafe impl<T: Sync + Send> Send for Arc<T> {}
unsafe impl<T: Sync + Send> Sync for Arc<T> {}

pub struct Weak<T> {
    ptr: NonNull<ArcData<T>>,
}

unsafe impl<T: Sync + Send> Send for Weak<T> {}
unsafe impl<T: Sync + Send> Sync for Weak<T> {}

impl<T> Arc<T> {
    #[allow(unused)]
    pub fn new(data: T) -> Arc<T> {
        Arc {
            ptr: NonNull::from(Box::leak(Box::new(ArcData {
                alloc_ref_count: AtomicUsize::new(1),
                data_ref_count: AtomicUsize::new(1),
                data: UnsafeCell::new(ManuallyDrop::new(data)),
            }))),
        }
    }

    fn data(&self) -> &ArcData<T> {
        unsafe { self.ptr.as_ref() }
    }

    #[allow(unused)]
    pub fn weak_count(&self) -> usize {
        unsafe { self.ptr.as_ref().alloc_ref_count.load(Ordering::Acquire) }
    }

    #[allow(unused)]
    pub fn strong_count(&self) -> usize {
        unsafe { self.ptr.as_ref().data_ref_count.load(Ordering::Acquire) }
    }

    #[allow(unused)]
    pub fn get_mut(arc: &mut Self) -> Option<&mut T> {
        // Acquire matches Weak::drop's Release decrement, to make sure any
        // upgraded pointers are visible in the next data_ref_count.load.
        // We do a spin loop in downgrade
        if arc
            .data()
            .alloc_ref_count
            .compare_exchange(1, usize::MAX, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            return None;
        }
        let is_unique = arc.data().data_ref_count.load(Ordering::Relaxed) == 1;
        // Release matches Acquire increment in `downgrade`, to make sure any
        // changes to the data_ref_count that come after `downgrade` don't
        // change the is_unique result above.
        arc.data().alloc_ref_count.store(1, Ordering::Release);
        if !is_unique {
            return None;
        }
        // Acquire to match Arc::drop's Release decrement, to make sure nothing
        // else is accessing the data.
        fence(Ordering::Acquire);
        unsafe { Some(&mut *arc.data().data.get()) }
    }

    #[allow(unused)]
    pub fn downgrade(arc: &Self) -> Weak<T> {
        let mut n = arc.data().alloc_ref_count.load(Ordering::Relaxed);
        loop {
            if n == usize::MAX {
                std::hint::spin_loop();
                n = arc.data().alloc_ref_count.load(Ordering::Relaxed);
                continue;
            }
            assert!(n <= usize::MAX >> 1);
            // Acquire synchronises with get_mut's release-store.
            if let Err(e) = arc.data().alloc_ref_count.compare_exchange_weak(
                n,
                n + 1,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                n = e;
                continue;
            }
            return Weak { ptr: arc.ptr };
        }
    }
}

impl<T> Deref for Arc<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.data().data.get() }
    }
}

impl<T> Weak<T> {
    fn data(&self) -> &ArcData<T> {
        unsafe { self.ptr.as_ref() }
    }

    #[allow(unused)]
    pub fn upgrade(&self) -> Option<Arc<T>> {
        let mut n = self.data().data_ref_count.load(Ordering::Relaxed);
        loop {
            if n == 0 {
                return None;
            }
            assert!(n <= usize::MAX >> 1);
            if let Err(e) = self.data().data_ref_count.compare_exchange_weak(
                n,
                n + 1,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                n = e;
                continue;
            }
            return Some(Arc { ptr: self.ptr });
        }
    }
}

impl<T> Clone for Weak<T> {
    fn clone(&self) -> Self {
        if self.data().alloc_ref_count.fetch_add(1, Ordering::Relaxed) > usize::MAX >> 1 {
            std::process::abort();
        }

        Weak { ptr: self.ptr }
    }
}

impl<T> Drop for Weak<T> {
    fn drop(&mut self) {
        if self.data().alloc_ref_count.fetch_sub(1, Ordering::Release) == 1 {
            fence(Ordering::Acquire);
            unsafe {
                drop(Box::from_raw(self.ptr.as_ptr()));
            }
        }
    }
}

impl<T> Clone for Arc<T> {
    fn clone(&self) -> Self {
        if self.data().data_ref_count.fetch_add(1, Ordering::Relaxed) > usize::MAX >> 1 {
            std::process::abort();
        }

        Arc { ptr: self.ptr }
    }
}

impl<T> Drop for Arc<T> {
    fn drop(&mut self) {
        if self.data().data_ref_count.fetch_sub(1, Ordering::Release) == 1 {
            fence(Ordering::Acquire);
            unsafe {
                ManuallyDrop::drop(&mut *self.data().data.get());
            }
        }

        drop(Weak { ptr: self.ptr });
    }
}

#[cfg(test)]
mod tests {
    use super::Arc;
    use std::{
        sync::atomic::{AtomicUsize, Ordering},
        thread,
    };

    static NUM_DROPS: AtomicUsize = AtomicUsize::new(0);

    struct DetectDrop;
    impl Drop for DetectDrop {
        fn drop(&mut self) {
            NUM_DROPS.fetch_add(1, Ordering::Relaxed);
        }
    }

    #[test]
    fn arc_is_cloned_downgradeable_and_upgradeable() {
        let arc_obj = Arc::new(("hello", DetectDrop));
        let arc_obj_clone_1 = Arc::downgrade(&arc_obj);
        let arc_obj_clone_2 = Arc::downgrade(&arc_obj);

        let t = thread::spawn(move || {
            // weak pointer is upgradeable
            let weak_ptr = arc_obj_clone_1.upgrade().unwrap();
            assert_eq!(weak_ptr.0, "hello");
        });

        assert_eq!(arc_obj.0, "hello");
        t.join().unwrap();

        // arc_obj_clone_1 has been dropped, but T still exists.
        assert_eq!(NUM_DROPS.load(Ordering::Relaxed), 0);
        assert!(arc_obj_clone_2.upgrade().is_some());

        drop(arc_obj);

        // main arc has been dropped, so T is deallocated, weak pointer cannot be upgraded anymore, but ArcData.data_ref_count and alloc_ref_count still exist
        assert_eq!(NUM_DROPS.load(Ordering::Relaxed), 1);
        assert!(arc_obj_clone_2.upgrade().is_none());
    }

    #[test]
    fn check_pointers_are_incrementing_correctly() {
        let arc_obj = Arc::new("hello");
        assert_eq!(*arc_obj, "hello");

        let _arc_obj_clone_1 = arc_obj.clone();
        assert_eq!(arc_obj.strong_count(), 2);

        // Arc internally clones weak to always ensure that ArcData always lives
        assert_eq!(arc_obj.weak_count(), 1);
    }

    #[test]
    fn check_pointers_are_decrementing_correctly() {
        let arc_obj = Arc::new("hello");
        assert_eq!(*arc_obj, "hello");

        let arc_obj_clone_1 = arc_obj.clone();
        assert_eq!(arc_obj.strong_count(), 2);

        // Arc internally clones weak to always ensure that ArcData always lives
        assert_eq!(arc_obj.weak_count(), 1);

        drop(arc_obj_clone_1);

        assert_eq!(arc_obj.strong_count(), 1);
        assert_eq!(arc_obj.weak_count(), 0);
    }
}
