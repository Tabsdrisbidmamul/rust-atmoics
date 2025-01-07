**Remember, memory ordering does not affect speed (whilst it may seem correlated it is not), it only defines when things happen. Hence, it is called memory ordering, and not memory timing or execution timing.**

# Relaxed

_Definition_: No synchronization or ordering guarantees. Operations happen at some point, but not necessarily in program order.

_Use_: When no ordering or visibility between threads is required, e.g., for performance counters

Example:

1. Thread A writes 1 to X (Relaxed).
2. Thread B reads X (Relaxed).
3. No guarantee B will see the updated value.

```rust
static X: AtomicI32 = AtomicI32::new(0);

let a = thread::spawn(|| {
  // No guarantee when the write happens
  X.store(1, Relaxed);
});

let b = thread::spawn(|| {
  // could see 0 or 1, unpredictable
  let value  = X.load(Relaxed)
});
```

# Release

_Definition_: Guarantees all writes in the current thread before a store(Release) are visible to other threads that perform a load(Acquire) on the same variable.

_Use_: Ensure changes are visible to other threads once a variable is set.

Example:

- Thread A stores 1 to X (Release).
- Thread B waits for X to be 1 (Acquire) before reading Y.

```rust
static X: AtomicI32 = AtomicI32::new(0);
static mut Y: i32 = 0;

let a = thread::spawn(|| {
  // Ensures that Y is set to 42 before X = 1;
  unsafe { Y = 42; }
  X.store(1, Release);
});

let b = thread::spawn(|| {
  // guarantees that Y is always 42
  if x.load(Acquire) == 1 {
    println!(Y);
  }
});
```

# Acquire

_Definition_: Ensures that all reads/writes in the current thread after load(Acquire) see the writes from a thread that performed a store(Release).

_Use_: Synchronize with a store(Release) to safely read shared data.

Example:

- Thread A stores 42 in Y
- Thread A stores 1 to X (Release).
- Thread B waits for X to be 1 (Acquire)

Same example as Release, but the ordering matters. Thread A must do writes before Release, so that thread B's Acquire will see all writes before Thread A's Release

If done the other way round, so writes are done after a Release and then Acquire is called. The variables will be uninitalised.

```rust
static X: AtomicI32 = AtomicI32::new(0);
static mut Y: i32 = 0;

let a = thread::spawn(|| {
  // Ensures that Y is set to 42 before X = 1;
  unsafe { Y = 42; }
  X.store(1, Release);
});

let b = thread::spawn(|| {
  // guarantees that Y is always 42
  if x.load(Acquire) == 1 {
    println!(Y);
  }
});
```

# SeqCst

_Definition_: Provides the strongest ordering guarantee. Ensures operations appear in the same order to all threads.

_Use_: When you need global ordering across all threads.

Example:

- Thread A stores 1 to X (SeqCst).
- Thread B reads X (SeqCst)

```rust
static X: AtomicI32 = AtomicI32::new(0);

let a = thread::spawn(|| {
  // guarantees order across all threads
  X.store(1, SeqCst);
});

let b = thread::spawn(|| {
  // guarantees to see 1
  let value = X.load(SeqCst);
});
```

From top to bottom, top being most performant but less synch, and bottom being least performant but most synch

| Ordering | Guarantees                                                                       | Use Case                                      |
| -------- | -------------------------------------------------------------------------------- | --------------------------------------------- |
| Relaxed  | No guarantees about visibility or ordering                                       | Performance counters, non-sync cases          |
| Release  | Writes before `store(Release)` are visible to `load(Acquire)` in other threads.  | Release shared data                           |
| Acquire  | Reads after `load(Acquire)` see writes from a `store(Release)` in other threads. | Synchronising with Release                    |
| SeqCst   | Global ordering of all atomic operations                                         | Simplest but slowest use case for correctness |

## Order of operations

As long as there is a Release and Acquire pair/ SeqCst. Then it doesn't matter how many Relaxed orderings you have, all threads will see the Relaxed updates when Release is stored, or Acquire is loaded.

# Fences

The last of memory ordering, this `std::sync::atomic::fence` provides memory constraint and ordering for non-atomic data. These act as a barrier for when mutating non-atomic data, and using an atomic flag.

The example below shows that we have an `AtomicBool` flag, and a non-atomic `Vec::<i32>`. We can't use a `Acquire` and `Release` as we did in `section_3/release_acquire.rs`. The fence does:

- Allows us to constrain and make certain that `reader` thread will see the change to `data` when `ready` is true

```rust
use std::sync::atomic::{AtomicBool, fence, Ordering};
use std::thread;

fn main() {
    // Shared flag and data
    let ready = AtomicBool::new(false);
    let mut data = vec![0; 10]; // non-atmoic data

    // Writer thread
    let writer = thread::spawn({
        let ready = &ready;
        move || {
            for i in 0..10 {
                data[i] = i + 1;
            }

            // Use a release fence to ensure writes to `data` are visible before updating `ready`
            fence(Ordering::Release);
            ready.store(true, Ordering::Relaxed);
        }
    });

    // Reader thread
    let reader = thread::spawn({
        let ready = &ready;
        move || {
            while !ready.load(Ordering::Acquire) {}

            // Use an acquire fence to ensure reads to `data` occur after `ready` is seen as true
            fence(Ordering::Acquire);
            println!("Data: {:?}", data);
        }
    });

    writer.join().unwrap();
    reader.join().unwrap();
}
```

# When to use Fences

## Summary

- Use fences for low-level, performance-critical scenarios where you can carefully control memory ordering and avoid race conditions.

- Use mutexes for general-purpose synchronization when simplicity, safety, and correctness are more important than raw performance.

The gist is that fences provided more control on memory ordering and are non-blocking. Great for high-performance applications.

Downside is that it is harder to read/ build a mental model on. Making it harder to debug data races.

Mutexes are simpler and guarantee consistency, but at the cost of system calls, context switching and kernel-level synchronisation.
