# Sequential Consistency (SeqCst)

- This allows global ordering between threads, so global variables (A, B, and S) will appear to other threads when they change atomically, and most importantly, in the same order.
- Lastly, all read and writes to SeqCst atomics are ordered relative to each other

Possible Outcomes:

1. Both Threads Race:

- If A.store(true, SeqCst) and B.store(true, SeqCst) happen concurrently, their effects might not be visible to the opposite thread immediately due to scheduling.
- Both !B.load(SeqCst) in thread a and !A.load(SeqCst) in thread b could evaluate to true, leading to both threads appending ! to S.

2. One Thread Completes First:

- If thread a executes fully before thread b, B will be false when a checks it, so S gets one !.
  Similarly, if thread b completes first, A will be false when b checks it, resulting in one !.

3. Sequential Execution:

- If thread a sets A and thread b sets B sequentially, the SeqCst ordering guarantees visibility of these updates. Neither thread will append ! to S, as they will observe the other variable already set.

## Why SeqCst Matters

- Without SeqCst (e.g., using Relaxed), there is no guarantee of ordering between threads, and the outcome becomes even more unpredictable.
- For instance, one thread might not see the other's store operation at all, even if it logically occurs earlier.

## Final Observations:

- The program may print S is "!!", S is "!", or S is "", depending on the interleaving of threads.
- Using SeqCst reduces but does not eliminate unpredictability due to the unsafe manipulation of S. Proper synchronization (e.g., a Mutex) would be necessary to ensure safety and determinism.
