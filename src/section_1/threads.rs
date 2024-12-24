use std::thread;

pub fn thread_main() {
    let t1 = thread::spawn(f);
    let t2 = thread::spawn(f);

    println!("Hello from the main thread");

    t1.join().unwrap();
    t2.join().unwrap();

    let numbers = Vec::from_iter(0..=100);

    let t = thread::spawn(move || {
        let len = numbers.len();
        let sum = numbers.iter().sum::<usize>();
        return sum / len;
    });

    let average = t.join().unwrap();
    println!("average {average}");

    let numbers = vec![1, 2, 3];

    thread::scope(|s| {
        s.spawn(|| {
            println!("length: {}", numbers.len());
            // numbers.push(1);
        });

        s.spawn(|| {
            for n in &numbers {
                println!("number {n}");
            }
        });
    });
}

fn f() {
    println!("Hello from another thread!");

    let id = thread::current().id();
    println!("This is my thread id: {id:?}");
}
