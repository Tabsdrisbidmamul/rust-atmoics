use std::{
    sync::{
        atomic::{AtomicU64, Ordering::Relaxed},
        OnceLock,
    },
    thread,
};

/**
 * lazy_init() is a naive approach to doing lazy initialisation, when 2 or more threads are trying to compute a value there can be a race condition where they both compute it.
 *
 * If the computing is a complicated and lengthy db/ network call, this can slow down the program.
 *
 * To get around that, Rust provides Once and OnceLock types which are thread safe, and provide an interface to initialising a value once.
 *
 * Once is used when you don't need to store a type, so useful for initialising a logger/ library.
 *
 * OnceLock is used when you need to store a value for later use.
 *
 * lazy_init_once_lock() will spawn 5 threads, and only one thread which gets the lock first will initialise the value and store it. The rest will simply retrieve the stored value. No race conditions anymore.
 *
 * Race condition: when the behaviour of the program relies on timing of interleaving events (such as lazy init), where the order of the threads can result in undesired outcomes. Can occur in single/ multi threaded programs
 *
 * Data races: specific type of race condition that occurs in multi-threaded programs, where 2 or more threads are trying to access shared memory. At lease one is a write. So bank account update and read being inconsistent.
 */

pub fn lazy_init() -> u64 {
    // default 0, but its non-zero value
    static VALUE: AtomicU64 = AtomicU64::new(0);
    let loader = VALUE.load(Relaxed);

    if loader == 0 {
        let calc_value = calculate_value();
        VALUE.store(calc_value, Relaxed);
    }

    return loader;
}

pub fn lazy_init_once_lock() {
    static INIT: OnceLock<u64> = OnceLock::new();
    let handles: Vec<_> = (0..5)
        .map(|_| {
            return thread::spawn(|| {
                INIT.get_or_init(|| {
                    let value = calculate_value();
                    println!("calculated value and initialised {value}");
                    return value;
                });
            });
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}

fn calculate_value() -> u64 {
    return 10;
}
