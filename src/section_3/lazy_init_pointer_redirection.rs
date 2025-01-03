use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};

#[derive(Debug)]
pub struct Data {
    #[allow(unused)]
    value: i32,
}

/**
 * Here we have done a "Acquire-Release" lock using Atomics compare-exchange. We set up an Atomic that stores a pointer (*mut T), and using Box type methods we can generate data on the heap and store the returned pointer back in the atomic.
 *
 * Using the compare-exchange, we can check if the initialised pointer (null) is still true, and store the pointer to the heap in the atomic, else drop the pointer and store the error instead.
 *
 * Lastly we return the pointer back to the caller.
 *
 * https://marabos.nl/atomics/images/raal_0305.png
 * p. 85 Rust Atomics
 *
 * This shows an illustration of the thread race and how the allocation and deallocation is working with Box.
 *
 * Here we can say that
 * - release ordering prevents the initialised data from being reordered with the store operation that shares the pointer to other threads.
 * - acquire ordering prevents reordering that would cause data to be accessed before the pointer is loaded
 *
 *
 */

pub fn get_pointer_data_lazy_init() -> &'static Data {
    static PTR: AtomicPtr<Data> = AtomicPtr::new(ptr::null_mut());

    let mut p = PTR.load(Ordering::Acquire);
    if p.is_null() {
        p = Box::into_raw(Box::new(generate_date()));
        if let Err(e) =
            PTR.compare_exchange(ptr::null_mut(), p, Ordering::Release, Ordering::Acquire)
        {
            // Error in compare-exchange, i.e p has already been initialised by another thread and is not null when doing the compare, so we drop pointer
            drop(unsafe { Box::from_raw(p) });
            p = e;
        }
    }

    // P is not null after the if block, and is initialised.
    return unsafe { &*p };
}

fn generate_date() -> Data {
    return Data { value: 2 };
}
