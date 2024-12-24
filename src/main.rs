#[allow(unused)]
use rust_atomics::section_1::{
    data_races_main, interior_mutability_cell_main, reference_counting_main, static_thread_main,
    thread_main,
};

fn main() {
    // section 1
    // thread_main();
    // static_thread_main()
    reference_counting_main();
    // data_races_main();
    // interior_mutability_cell_main();
}
