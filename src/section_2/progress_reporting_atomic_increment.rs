use std::{
    sync::atomic::{AtomicI32, Ordering::Relaxed},
    thread,
    time::Duration,
};

/**
 * We have 4 threads batch processing 25 items, so each thread doing a 1/4 of workload in parallel.
 *
 * 3 key points:
 * 1. We now use a reference for num_done, instead of by value. We do this because we need a shared reference for all 4 threads to reference back to.
 * 2.
 * -  We use capture the num_done variable in the each of the spawned threads closure.
 * -  Rust needs this to enforce that the each thread have their own clone copy of the reference, in case the main thread exits out early, the 4 threads have their copy to reference, ensuring there are no dangling pointers to the main num_done pointer if the pointer is dropped if the main fn exits.
 * 3. The move also moves the value t in the wrapping loop of the thread::spawn. It can't do this by reference do copying the int from stack to the thread prevents dangling pointers and stack values if dropped when main fn exits
 *
 *the fetch_add() will increment the value stored in the Atomic, and return the old value if needed. So all 4 threads are incrementing the pointer (behind the scenes I imagine a lock is in place for writes).
 *
 * The main thread can read the Atomic, and update the UI with the updated values.
 *
 */

pub fn progress_reporting_increment() {
    let num_done = &AtomicI32::new(0);

    thread::scope(|s| {
        // four background threads to process 100 items, 25 each
        for t in 0..4 {
            s.spawn(move || {
                for i in 0..25 {
                    process_item(t * 25, i);
                    num_done.fetch_add(1, Relaxed);
                }
            });
        }

        // main thread shows status updates, every second
        loop {
            let n = num_done.load(Relaxed);
            if n == 100 {
                break;
            }
            println!("Working...{n}/100 done");
            thread::sleep(Duration::from_secs(1));
        }
    });

    println!("done");
}

fn process_item(_: i32, _: i32) -> () {
    thread::sleep(Duration::from_secs(1));
}
