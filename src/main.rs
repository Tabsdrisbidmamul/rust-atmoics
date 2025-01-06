#[allow(unused)]
use rust_atomics::section_1::{
    data_races_main, interior_mutability_cell_main, mutex_guard, reference_counting_main,
    static_thread_main, thread_condvar_mutex, thread_main, thread_parking,
};

#[allow(unused)]
use rust_atomics::section_2::{
    lazy_init, lazy_init_once_lock, progress_reporting, progress_reporting_increment,
    statistics_progress, stop_atomic_main,
};

#[allow(unused)]
use rust_atomics::section_3::{
    fence_sitting, get_pointer_data_lazy_init, release_acquire_example, seq_cst_ordering,
};

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
    // stop_atomic_main();
    // progress_reporting();
    // lazy_init();
    // lazy_init_once_lock();
    // progress_reporting_increment();
    // statistics_progress();

    // ------section 3------
    // release_acquire_example();
    // dbg!(get_pointer_data_lazy_init());
    // seq_cst_ordering();
    // fence_sitting();
}
