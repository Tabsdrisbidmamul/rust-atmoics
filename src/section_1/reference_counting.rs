use std::{rc::Rc, sync::Arc, thread};

pub fn reference_counting_main() {
    let a = Rc::new([1, 2, 3]);
    let b = a.clone();

    assert_eq!(a.as_ptr(), b.as_ptr());

    /*Rc are not thread safe and not atomic, instead use Arc which is thread safe and atomic */
    // thread::spawn(move || dbg!(b));

    let a = Arc::new([1, 2, 3]);
    let b = a.clone();

    // both threads get their own Arc (so there are 4 Arcs that all point to the same array)
    let t1 = thread::spawn(move || dbg!(a));
    let t2 = thread::spawn(move || dbg!(b));

    t1.join().unwrap();
    t2.join().unwrap();

    // last thread to see Arc go to 0 will drop and deallocate memory
}
