use std::{
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::{Duration, Instant},
};

/**
 * Each thread will start a timer to measure process_item() being 1 second sleep on the thread.
 *
 * So we measure the time and add the time taken to the cumulative total_time, and max_time is the maximum of any thread time taken.
 *
 * The 4 threads will update the total_time and max_time as num_done is incrementing. What this means is that the main thread may load in an out of date total_time when taking the average. So we might see in the terminal inconsistent averages.
 *
 * A mutex can sort out this issue, but we lose out on the Atomic ops and we now have to consider blocking threads, locking and unlocking a mutex.
 */

pub fn statistics_progress() {
    let num_done = &AtomicU64::new(0);
    let total_time = &AtomicU64::new(0);
    let max_time = &AtomicU64::new(0);

    thread::scope(|s| {
        for t in 0..4 {
            s.spawn(move || {
                for i in 0..25 {
                    let start = Instant::now();
                    process_item(t * 25, i);
                    let time_taken = start.elapsed().as_micros() as u64;
                    num_done.fetch_add(1, Ordering::Relaxed);
                    total_time.fetch_add(time_taken, Ordering::Relaxed);
                    max_time.fetch_max(time_taken, Ordering::Relaxed);
                }
            });
        }

        loop {
            let total_time = Duration::from_micros(total_time.load(Ordering::Relaxed));
            let max_time = Duration::from_micros(max_time.load(Ordering::Relaxed));

            let n = num_done.load(Ordering::Relaxed);
            if n == 100 {
                break;
            }

            if n == 0 {
                println!("Working.. nothing done yet.");
            } else {
                println!(
                    "Working..{n}/100 done, {:?} average, {:?} peak",
                    total_time / n as u32,
                    max_time
                );
            }
            thread::sleep(Duration::from_secs(1));
        }
    });

    println!("done");
}

fn process_item(_: i32, _: i32) {
    thread::sleep(Duration::from_secs(1));
}
