use std::cell::Cell;

pub fn interior_mutability_cell_main() {
    interior_mutability_cell(&Cell::new(6), &Cell::new(5));
}

/**
 * The refs a and b could be pointing to the same value (its a boxed pointer), so the if block may come true and the compiler allows this
 */
fn interior_mutability_cell(a: &Cell<i32>, b: &Cell<i32>) {
    let before = a.get();
    b.set(b.get() + 1);

    let after = a.get();
    if before != after {
        println!("might happen");
    }
}

#[allow(unused)]
/**
 * We cannot modify the cell directly, but take the contents of the cell, modify it and assign the new value back to the cell.
 */
fn set_values_in_cell(v: &Cell<Vec<i32>>) {
    let mut v2 = v.take();
    v2.push(1);
    v.set(v2);
}
