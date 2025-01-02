use std::{
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    thread,
    time::Duration,
};

/**
 * The "happens-before" relationship
 * We can say that for
 * let x = 5;
 * let y = 10;
 *
 * that x is loaded in with 5 first, then y is loaded with 10 second on a single thread.
 *
 * But on n threads using a Relaxed Memory model, we cannot guarantee that x is loaded first or y is loaded second. Memory is loaded in an order best optimised by the compiler
 *
 * Release-Acquire memory ordering works like this:
 * - The producer writes to READY with Release ensures that threads watching it will see all changes before it (this includes the write to DATA with Relaxed).
 *
 * - This guarantees that when reading READY with Acquire ordering, we can see all consumer changes and this includes DATA.
 *
 * So the while loop checking for true with Acquire will see the change to DATA, so when we print the final result it will always be 123, because we have told the compiler to order memory in such a way (reading the program top to bottom) that the 123 is guaranteed to be stored and available when read with Relaxed after the while loop blocking process.
 *
 * If this was all Relaxed, the while loop may see the READY as true, but the thread is still writing 123 to DATA, so when we print the final result may be 0, and some time after it has written 123.
 *
 * We have effectively ensured threads are processing memory in a correct order for the program to do "business logic".
 */

pub fn release_acquire_example() {
    static DATA: AtomicU64 = AtomicU64::new(0);
    static READY: AtomicBool = AtomicBool::new(false);

    thread::spawn(|| {
        DATA.store(123, Ordering::Relaxed);
        READY.store(true, Ordering::Release);
    });

    while !READY.load(Ordering::Acquire) {
        thread::sleep(Duration::from_secs(1));
        println!("waiting...");
    }

    println!("{}", DATA.load(Ordering::Relaxed));
}
