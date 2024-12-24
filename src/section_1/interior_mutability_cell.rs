use std::cell::Cell;

pub fn interior_mutability_cell_main() {
    interior_mutability_cell(&Cell::new(6), &Cell::new(5));
}

fn interior_mutability_cell(a: &Cell<i32>, b: &Cell<i32>) {
    let before = a.get();
    b.set(b.get() + 1);

    let after = a.get();
    if before != after {
        println!("might happen");
    }
}
