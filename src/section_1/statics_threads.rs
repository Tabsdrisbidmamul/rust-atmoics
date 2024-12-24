use std::thread;

pub fn static_thread_main() {
    static X: [i32; 3] = [1, 2, 3];

    thread::spawn(|| dbg!(&X));
    thread::spawn(|| dbg!(&X));

    let x: &'static [i32; 3] = Box::leak(Box::new([1, 2, 3]));

    thread::spawn(move || dbg!(x));
    thread::spawn(move || dbg!(x));
}
