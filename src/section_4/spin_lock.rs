use std::{
    cell::UnsafeCell,
    sync::atomic::{AtomicBool, Ordering},
};

/**
 * Spin lock Mutex which will allows threads to keep pinging the lock till its free.
 */
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

    pub fn lock(&self) -> &mut T {
        while self.locked.swap(true, Ordering::Acquire) {
            std::hint::spin_loop();
        }

        return unsafe { &mut *self.value.get() };
    }

    /// Safety: the &mut T from lock() must be dropped when before calling unlock.
    pub unsafe fn unlock(&self) {
        self.locked.store(false, Ordering::Release);
    }
}
