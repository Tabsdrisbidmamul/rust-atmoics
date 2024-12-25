use std::{sync::Mutex, thread};

/**
 * A typical mutex has methods lock() and unlock(), but in rust there is only lock(). This returns a guard, which guarantees that when dropped (dealloc) that the lock will on the mutex will be unlocked and the next thread in the pool will have access to the the mutex lock
 *
 * The mutex wraps a value T, and can be accessed through deref (*), only when a thread has a lock on the mutex can this be done.
 *
 * This ensures that the value wrapped by the mutex is protected from other threads unless they have the lock
 */
pub fn mutex_guard() {
    let n = Mutex::new(0);
    thread::scope(|s| {
        for _ in 0..10 {
            s.spawn(|| {
                let mut guard = n.lock().unwrap();
                for _ in 0..100 {
                    *guard += 1;
                }
            });
        }
    });

    // into_inner takes ownership of the mutex, so locks are unnecessary and the main thread has taken over.
    assert_eq!(n.into_inner().unwrap(), 1000);
    println!("mutex inner value is 1000");
}
