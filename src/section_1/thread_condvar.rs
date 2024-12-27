use std::{
    collections::VecDeque,
    sync::{Condvar, Mutex},
    thread,
    time::Duration,
};

/**
 * Condvar (condition variable) is a way to signal between threads that an event has occurred for those threads that are waiting on that condition.
 *
 * This means that threads are blocked till a condition has occurred, but the blocked threads do not consume CPU time, simply occupy memory for said thread.
 *
 * This is a step up on park and unpark, as that can be spurious. The the unparked thread will wake up to find that the condition is unsatisfied (in our previous case, the queue is still empty).
 *
 * A Condvar is meant to be used with a Mutex. The Condvar::wait will release the lock and put the thread to sleep.
 *
 * When the producer notifies a single or n threads to wake up, it will do a spurious wakeup check (check that the condition has been met). We have a check with the if block when popping from the front to see if a value comes back.
 *
 * Unlocking, waiting, and relocking are all done with the wait() method.
 * We still have a lock on the queue, so when wait is called,
 * - The thread releases its lock on the queue
 * - Its put into waiting mode (sleep)
 * - When the sleeping thread is notified, it will attempt to get a lock on the Mutex (other threads may be contending, depends on OS scheduler which thread gets the lock)
 *
 * wait_timeout() and park_timeout() methods take a duration, and when the time has elapsed and the thread was not woken by a notification or unpark(). The thread will wake up and be either satisfied or unsatisfied with the Mutex state (this is the spurious, and why its difficult)
 */

pub fn thread_condvar_mutex() {
    let queue = Mutex::new(VecDeque::<i32>::new());
    let not_empty = Condvar::new();

    thread::scope(|s| {
        // Consuming thread
        s.spawn(|| loop {
            let mut q = queue.lock().unwrap();
            let item = loop {
                if let Some(item) = q.pop_front() {
                    break item;
                } else {
                    q = not_empty.wait(q).unwrap();
                }
            };
            // If all is well, we need to drop the lock,in the case of other threads, can get the lock on the Mutex
            drop(q);
            dbg!(item);
        });

        // Producing thread
        for i in 0..=10 {
            queue.lock().unwrap().push_back(i);
            not_empty.notify_one();
            thread::sleep(Duration::from_secs(1));
        }
    });
}
