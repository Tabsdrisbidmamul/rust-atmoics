use std::{
    cell::UnsafeCell,
    ops::Deref,
    ptr::NonNull,
    sync::atomic::{fence, AtomicUsize, Ordering},
};

struct ArcData<T> {
    // pointers to T
    strong_ref_count: AtomicUsize,

    // pointers to ArcData
    weak_ref_count: AtomicUsize,

    // Data T. None if there is only one weak pointer left
    data: UnsafeCell<Option<T>>,
}

/**
 * Weak is keeping the Arc object alive for this program, Weak is the center point for Arc. Weak contains the pointer to ArcData and is the one that implements Send and Sync, thus Arc also implements Send and Sync.
 *
 * A weak pointer is a non-owning reference, it simply observes the data, but cannot access or mutate it.
 *
 * To get ownership it needs to be upgraded to become an Arc. If the data it is observing no longer exists, it cannot be upgraded, returns None/ null as you would expect from a deallocated part of memory.
 *
 * If no arc pointers exist (strong pointers) T is de-allocated. But if weak pointer exists, the reference count will still exist.
 *
 *
 */

pub struct Arc<T> {
    weak: Weak<T>,
}

pub struct Weak<T> {
    ptr: NonNull<ArcData<T>>,
}

unsafe impl<T: Sync + Send> Send for Weak<T> {}
unsafe impl<T: Sync + Send> Sync for Weak<T> {}

impl<T> Arc<T> {
    #[allow(unused)]
    pub fn new(data: T) -> Arc<T> {
        Arc {
            weak: Weak {
                ptr: NonNull::from(Box::leak(Box::new(ArcData {
                    weak_ref_count: AtomicUsize::new(1),
                    strong_ref_count: AtomicUsize::new(1),
                    data: UnsafeCell::new(Some(data)),
                }))),
            },
        }
    }

    #[allow(unused)]
    pub fn weak_count(&self) -> usize {
        unsafe {
            self.weak
                .ptr
                .as_ref()
                .weak_ref_count
                .load(Ordering::Relaxed)
        }
    }

    #[allow(unused)]
    pub fn strong_count(&self) -> usize {
        unsafe {
            self.weak
                .ptr
                .as_ref()
                .strong_ref_count
                .load(Ordering::Relaxed)
        }
    }

    #[allow(unused)]
    pub fn get_mut(arc: &mut Self) -> Option<&mut T> {
        if arc.weak.data().strong_ref_count.load(Ordering::Relaxed) > 1 {
            return None;
        }

        // There is only one pointer referencing Arc, so we can `safely` get the &mut pointer, and we know that data has not been dropped.
        fence(Ordering::Acquire);
        let arc_data = unsafe { arc.weak.ptr.as_mut() };
        let option = arc_data.data.get_mut();
        let data = option.as_mut().unwrap();
        Some(data)
    }

    ///
    /// No need to decrement the data_ref_count, this is a constructor method which will take in an Arc and create a Weak clone.
    ///
    #[allow(unused)]
    pub fn downgrade(arc: &Self) -> Weak<T> {
        arc.weak.clone()
    }
}

impl<T> Weak<T> {
    fn data(&self) -> &ArcData<T> {
        unsafe { self.ptr.as_ref() }
    }

    ///
    /// Will upgrade Weak pointer to an Arc, if T exists
    /// We do this by checking data_ref_count is greater than 0, and increment the count and return an Arc
    ///
    #[allow(unused)]
    pub fn upgrade(&self) -> Option<Arc<T>> {
        let mut node = self.data().strong_ref_count.load(Ordering::Relaxed);
        loop {
            if node == 0 {
                return None;
            }
            assert!(node <= usize::MAX >> 1);

            if let Err(e) = self.data().strong_ref_count.compare_exchange_weak(
                node,
                node + 1,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                node = e;
                continue;
            }

            return Some(Arc { weak: self.clone() });
        }
    }
}

impl<T> Deref for Arc<T> {
    type Target = T;

    fn deref(&self) -> &T {
        let ptr = self.weak.data().data.get();
        unsafe { (*ptr).as_ref().unwrap() }
    }
}

/**
 * Weak is cloned through Arc::clone(), same code as from `section_5/arc_basic` but Weak increments alloc_ref_count (data + weak ptr)
 *
 * Arc increments data_ref_count
 */

impl<T> Clone for Weak<T> {
    fn clone(&self) -> Self {
        if self.data().weak_ref_count.fetch_add(1, Ordering::Relaxed) > usize::MAX >> 1 {
            std::process::abort();
        }

        Weak { ptr: self.ptr }
    }
}

impl<T> Clone for Arc<T> {
    fn clone(&self) -> Self {
        let weak = self.weak.clone();
        if weak.data().strong_ref_count.fetch_add(1, Ordering::Relaxed) > usize::MAX >> 1 {
            std::process::abort();
        }

        Arc { weak }
    }
}

///
/// Will drop ArcData struct
///
impl<T> Drop for Weak<T> {
    fn drop(&mut self) {
        if self.data().weak_ref_count.fetch_sub(1, Ordering::Relaxed) == 1 {
            fence(Ordering::Acquire);
            unsafe { drop(Box::from_raw(self.ptr.as_ptr())) }
        }
    }
}

///
/// Will deallocate data T
///
impl<T> Drop for Arc<T> {
    fn drop(&mut self) {
        if self
            .weak
            .data()
            .strong_ref_count
            .fetch_sub(1, Ordering::Relaxed)
            == 1
        {
            fence(Ordering::Acquire);
            let ptr = self.weak.data().data.get();
            unsafe {
                (*ptr) = None;
            }
        }
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
    fn check_arc_is_created_and_dropped() {
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
        assert_eq!(arc_obj.weak_count(), 2);
    }

    #[test]
    fn check_pointers_are_decrementing_correctly() {
        let arc_obj = Arc::new("hello");
        assert_eq!(*arc_obj, "hello");

        let arc_obj_clone_1 = arc_obj.clone();
        assert_eq!(arc_obj.strong_count(), 2);

        // Arc internally clones weak to always ensure that ArcData always lives
        assert_eq!(arc_obj.weak_count(), 2);

        drop(arc_obj_clone_1);

        assert_eq!(arc_obj.strong_count(), 1);
        assert_eq!(arc_obj.weak_count(), 1);
    }
}
