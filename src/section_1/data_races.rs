pub fn data_races_main() {
    f(&5, &mut 10);
}

fn f(a: &i32, b: &mut i32) {
    let before = *a;
    *b += 1;

    let after = *a;
    if before != after {
        println!("never happens");
    }
}
