#[allow(unused)]
use rust_atomics::section_1::{
    data_races_main, interior_mutability_cell_main, mutex_guard, reference_counting_main,
    static_thread_main, thread_condvar_mutex, thread_main, thread_parking,
};

#[allow(unused)]
use rust_atomics::section_2::stop_atomic_main;

fn main() {
    // ------section 1------
    // thread_main();
    // static_thread_main()
    // reference_counting_main();
    // data_races_main();
    // interior_mutability_cell_main();
    // mutex_guard();
    // thread_parking();
    // thread_condvar_mutex();

    // ------section 2------
    stop_atomic_main();
}
