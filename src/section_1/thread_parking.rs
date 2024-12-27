use std::{collections::VecDeque, sync::Mutex, thread, time::Duration};

/**
 * Thread parking is a technique that allows us to put a thread to sleep (so park it), do the thread stops consuming CPU time, but still occupies a place in memory. This is useful as spawning a thread by asking the OS all the time is expensive, so by creating a pool of pre-allocated threads but park it and unpark when needed is a quick way to get those threads to do work.
 *
 * main thread -> parking thread master -> [worker threads of n count]
 *
 * - The main thread is the process thread that runs the program.
 * - The parking thread master can be another thread or the main thread.
 * - The worker threads can be of n count that
 *
 * This example shows that we can keep a ref of a worker thread, dequeue an item and process, and when the queue is empty, the thread parks itself.
 *
 * The producer can keep a ref of all worker threads or a handler that will keep track, and when the producer enqueues an item, unpark threads from worker pool and the thread will do its work (like a callback)
 */

pub fn thread_parking() {
    let queue = Mutex::new(VecDeque::<i32>::new());

    thread::scope(|s| {
        // Consuming thread
        let t = s.spawn(|| loop {
            let item = queue.lock().unwrap().pop_front();
            if let Some(item) = item {
                dbg!(item);
            } else {
                thread::park();
            }
        });

        // Producing thread
        for i in 0..=10 {
            queue.lock().unwrap().push_back(i);
            t.thread().unpark();
            thread::sleep(Duration::from_secs(1));
        }
    });
}
