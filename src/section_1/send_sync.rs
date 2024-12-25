#[allow(unused)]
use std::{cell::Cell, marker::PhantomData, rc::Rc, sync::Arc, thread};

#[allow(unused)]
/**
 * Send: indicates ownership of a value can be safely transferred (moved) between threads. so i32, bool, etc
 *
 * Sync: indicates that a type is safe and be can be referenced from multiple threads simultaneously. so Arc or if T is sync, then &T can be shared between threads
 */

/**
 * If handle was the only field, the struct would have been both Send and Sync. But with a zero-sized field, which is treated as Cell<()>, which is Send only. So the struct takes the least implemented and is Send.
 *
 * *const T and *mut are neither Send or Sync, not enough information for compiler - so you will have to implement those traits yourself.
 *
 * PhantomData does not exist at runtime, so it takes no space
 */
struct X {
    handle: i32,
    _not_sync: PhantomData<Cell<()>>,
}

/**
 * This is discouraged to do, unless you know what you are doing, but you can impl Send and Sync for types without inference through types.
 */
unsafe impl Send for X {}
unsafe impl Sync for X {}

#[allow(unused)]
fn send_type_but_fail() {
    // Rc does not impl Send, so you cannot move the value to another thread, so it will fail

    // Arc impl Send, so it can be moved to another thread

    // thread::spawn requires its argument and values that it captures to be Send
    let a = Arc::new(123);
    thread::spawn(move || {
        dbg!(a);
    });
}
