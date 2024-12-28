use std::{
    sync::atomic::{AtomicI32, Ordering::Relaxed},
    thread,
    time::Duration,
};

/**
 * Progress reporter example, this spawns and joins a bg thread which will process some function and store the progress in an AtomicI32.
 *
 * The main thread will continually loop and retrieve back the stored AtomicI32 from the bg thread and output the result to stdout.
 *
 * We use park() and unpark() to wake the main thread up when the bg thread has completed its work. We do this where the main thread is asleep and may result pulling back an inconsistent progress
 *
 */

pub fn progress_reporting() {
    let num_done = AtomicI32::new(0);
    let main_thread = thread::current();

    thread::scope(|s| {
        // A background thread to process all 10 items
        s.spawn(|| {
            for i in 0..10 {
                process_item(i);
                num_done.store(i + 1, Relaxed);
                main_thread.unpark();
            }
        });

        loop {
            let n = num_done.load(Relaxed);
            if n == 10 {
                break;
            }
            println!("Working...{n}/10 done");
            thread::park_timeout(Duration::from_secs(1));
        }
    });

    println!("done");
}

fn process_item(_value: i32) {
    thread::sleep(Duration::from_secs(1));
}
