use std::{
    ops::Deref,
    ptr::NonNull,
    sync::atomic::{fence, AtomicUsize, Ordering},
};

#[derive(Debug)]
struct ArcData<T> {
    ref_count: AtomicUsize,
    data: T,
}

/**
 * Using NonNull ptr, we are telling the compiler that Arc<T> is not either Send or Sync, but it will have to be implemented for each case. In our case we need our Arc to be both Send and Sync:
 * Sync:
 * - We are sharing the reference T between threads
 *
 * Send:
 * - We are copying the value T between threads (of which said thread can drop T, so it will need its own copy)
 */

#[derive(Debug)]
pub struct Arc<T> {
    ptr: NonNull<ArcData<T>>,
}

unsafe impl<T: Send + Sync> Send for Arc<T> {}
unsafe impl<T: Send + Sync> Sync for Arc<T> {}

impl<T> Arc<T> {
    ///
    /// Will create a new Arc object, in which the ptr is set to a NonNull pointer, which is leaked from Box::leak, by storing our ArcData on the heap via Box::new(). ArcData will initialise its ref count to 1 and store data within it.
    ///
    ///
    #[allow(unused)]
    pub fn new(data: T) -> Arc<T> {
        Arc {
            ptr: NonNull::from(Box::leak(Box::new(ArcData {
                ref_count: AtomicUsize::new(1),
                data,
            }))),
        }
    }

    ///
    /// The compiler is not able to verify that the ptr is pointer to a value. Even though its constructed, the compiler is unaware because of NonNull. But we know its there, since it has to be constructed via new()
    ///
    fn data(&self) -> &ArcData<T> {
        unsafe { self.ptr.as_ref() }
    }

    #[allow(unused)]
    pub fn count(&self) -> usize {
        unsafe { self.ptr.as_ref().ref_count.load(Ordering::Relaxed) }
    }

    ///
    /// Will return a mutable pointer if there exists only one pointer to T
    ///
    #[allow(unused)]
    pub fn get_mut(arc: &mut Self) -> Option<&mut T> {
        if arc.data().ref_count.load(Ordering::Relaxed) > 1 {
            return None;
        }

        fence(Ordering::Acquire);
        unsafe { Some(&mut arc.ptr.as_mut().data) }
    }
}

/**
 * We cannot implement DerefMut, since T in Arc<T> is shared ownership so &T. &mut T is exclusive ownership, i.e. only one owner at one given time
 */

impl<T> Deref for Arc<T> {
    type Target = T;

    ///
    /// *Arc<T> will return back T
    ///
    fn deref(&self) -> &T {
        &self.data().data
    }
}

/**
 * Increment the AtomicUsize pointer by one via Relaxed memory constraint, since there is no other operations that need to happen before or after this.
 *
 * We do an if check to see if the ref_count is not greater than []usize::MAX (2^64 -1) / 2].
 *
 * We divide by 2, using bit shift op since divide is costly, but we do this to stop an overflow occurring when we call abort(). The instruction abort() does take to execute, and in that time another thread may call clone() and we get an overflow if we did usize - 1.
 *
 * So by dividing by 2 we we are sure that the abort instruction will complete before another thread calls clone().
 */

impl<T> Clone for Arc<T> {
    fn clone(&self) -> Self {
        if self.data().ref_count.fetch_add(1, Ordering::Relaxed) > usize::MAX >> 1 {
            std::process::abort();
        }

        Arc { ptr: self.ptr }
    }
}

/**
 * Decrement the ref counter, and the thread that sees it go to zero (fetch_sub returns the previous value, so it being 1 means that the ref_count is 0) will drop ArcData
 *
 * For memory ordering, we need to make sure that nothing is still accessing the data, so we use an Acquire fence to handle the memory ordering before we drop.
 *
 * fetch_sub can remain Relaxed, the fence handles the memory ordering
 *
 */

impl<T> Drop for Arc<T> {
    fn drop(&mut self) {
        if self.data().ref_count.fetch_sub(1, Ordering::Relaxed) == 1 {
            fence(Ordering::Acquire);
            unsafe { drop(Box::from_raw(self.ptr.as_ptr())) }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::atomic::{AtomicUsize, Ordering},
        thread,
    };

    use super::Arc;

    static NUM_DROPS: AtomicUsize = AtomicUsize::new(0);

    struct DetectDrop;
    impl Drop for DetectDrop {
        fn drop(&mut self) {
            NUM_DROPS.fetch_add(1, Ordering::Relaxed);
        }
    }

    #[test]
    fn arc_is_created_and_dropped() {
        let arc_obj = Arc::new(("hello", DetectDrop));
        let arc_obj_clone = arc_obj.clone();

        let t = thread::spawn(move || {
            assert_eq!(arc_obj.0, "hello");
        });

        assert_eq!(arc_obj_clone.0, "hello");

        t.join().unwrap();

        // x has been dropped, this test checks if the object data hasn't been dropped.
        assert_eq!(NUM_DROPS.load(Ordering::Relaxed), 0);

        drop(arc_obj_clone);

        // y has been dropped, so we check if the object data has been dropped too
        assert_eq!(NUM_DROPS.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn arc_ref_count_is_incrementing_correctly() {
        let arc_obj = Arc::new("hello");

        assert_eq!(*arc_obj, "hello");
        assert_eq!(arc_obj.count(), 1);

        let _arc_obj_clone = arc_obj.clone();

        assert_eq!(arc_obj.count(), 2);
    }

    #[test]
    fn arc_ref_count_is_decrementing_correctly() {
        let arc_obj = Arc::new("hello");

        assert_eq!(*arc_obj, "hello");
        assert_eq!(arc_obj.count(), 1);

        let arc_obj_clone = arc_obj.clone();
        assert_eq!(arc_obj.count(), 2);

        drop(arc_obj_clone);
        assert_eq!(arc_obj.count(), 1);
    }
}
