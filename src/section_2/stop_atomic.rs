use std::{
    sync::atomic::{AtomicBool, Ordering},
    thread,
};

/**
 * Atomic operations is some task or process that has been complete or not at all. Like transactions.
 *
 * We use an AtomicBool as a flag between our main and worker threads to signal to the worker thread to do something based on the flag.
 *
 * The AtomicBool has 2 methods, store and load, each taking a shared ref (&T), store does not take a exclusive ref (&mut T). It utilizes the interior mutability we covered in section 1 Cells.
 *
 * Since its atomic, the store/ load will always store or load the arguments (and not).
 */

pub fn stop_atomic_main() {
    static STOP: AtomicBool = AtomicBool::new(false);

    // spawn thread to do work
    let background_thread = thread::spawn(|| {
        while !STOP.load(Ordering::Relaxed) {
            some_work()
        }
    });

    // use main thread to listen for user input
    for line in std::io::stdin().lines() {
        match line.unwrap().as_str() {
            "help" => println!("commands: help, stop"),
            "stop" => break,
            cmd => println!("unknown command: {cmd:?}"),
        }
    }

    // inform the bg thread it needs to stop
    STOP.store(true, Ordering::Relaxed);
    background_thread.join().unwrap();
}

fn some_work() {
    //
}
