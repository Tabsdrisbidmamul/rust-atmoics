use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, Ordering},
    thread,
};

/**
 * Spin lock Mutex which will allows threads to keep pinging the lock till its free.
 */
#[derive(Debug)]
pub struct SpinLock<T> {
    locked: AtomicBool,
    value: UnsafeCell<T>,
}

unsafe impl<T> Sync for SpinLock<T> where T: Send {}

/**
 * We use Release-Acquire store to ensure that all threads will see.
 *
 * swap will store the value in the parameter and return the previous value to the caller
 */
impl<T> SpinLock<T> {
    pub const fn new(value: T) -> Self {
        return Self {
            locked: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        };
    }

    ///
    /// Returns a Deref/ DerefMut of Guard, so the caller has access to the protected value T, and operate on it as normal.
    ///
    pub fn lock(&self) -> Guard<T> {
        // until the lock is false i.e. unlock state, do we only return back the Guard and set the lock back to locked.
        while self.locked.swap(true, Ordering::Acquire) {
            std::hint::spin_loop();
        }

        return Guard { lock: self };
    }

    // Guard's drop handles unlocking
    // Safety: the &mut T from lock() must be dropped when before calling unlock.
    // pub unsafe fn unlock(&self) {
    //     self.locked.store(false, Ordering::Release);
    // }
}

#[derive(Debug)]
pub struct Guard<'a, T> {
    lock: &'a SpinLock<T>,
}
/**
 * Implement Deref trait so *n will give us back the actual value stored at address n.
 *
 * The lock() method returns back a Guard, the Deref will work on this and return back &T or &mut T to the caller when they call lock(), allowing them access to the value.
 *
 * The code is an unsafe block, but with the existence of the guard we can be sure that there is only one lock to a thread, so we can assume from that, and deref the value.
 */

impl<T> Deref for Guard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        // Safety: the existence of Guard guarantees that we have exclusively locked the lock
        unsafe { &*self.lock.value.get() }
    }
}

impl<T> DerefMut for Guard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // Safety: the existence of Guard guarantees that we have exclusively locked the lock
        unsafe { &mut *self.lock.value.get() }
    }
}

/**
 * When guard goes out of scope, unlock the lock. No longer need the unlock method in SpinLock. The user will drop the Guard. Also means that the user does not need to handle unsafe code
 */

impl<T> Drop for Guard<'_, T> {
    fn drop(&mut self) {
        self.lock.locked.store(false, Ordering::Release)
    }
}

pub fn spin_lock_main() {
    let spin_lock = SpinLock::new(Vec::<i32>::new());
    thread::scope(|s| {
        s.spawn(|| {
            spin_lock.lock().push(1);
        });

        s.spawn(|| {
            let mut guard = spin_lock.lock();
            guard.push(2);
            guard.push(3);
        });
    });

    let guard = spin_lock.lock();
    dbg!(&guard);
    dbg!(&guard.as_slice());
    assert!(guard.as_slice() == [1, 2, 3] || guard.as_slice() == [2, 3, 1])
}
