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
