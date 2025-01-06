use rand::{rngs::ThreadRng, Rng};
use std::{
    sync::atomic::{fence, AtomicBool, Ordering},
    thread,
    time::Duration,
};

static mut DATA: [u64; 10] = [0; 10];

const ATOMIC_FALSE: AtomicBool = AtomicBool::new(false);
static READY: [AtomicBool; 10] = [ATOMIC_FALSE; 10];

pub fn fence_sitting() {
    for i in 0..10 {
        thread::spawn(move || {
            let mut rng = rand::thread_rng();
            let data = some_calculation(&mut rng);
            unsafe {
                DATA[i] = data;
            }
            READY[i].store(true, Ordering::Release);
        });
    }

    thread::sleep(Duration::from_millis(500));
    let ready: [bool; 10] = std::array::from_fn(|i| READY[i].load(Ordering::Relaxed));

    if ready.contains(&true) {
        fence(Ordering::Acquire);
        for i in 0..10 {
            if ready[i] {
                println!("data[{i}] = {}", unsafe { DATA[i] });
            }
        }
    }
}

fn some_calculation(rng: &mut ThreadRng) -> u64 {
    return rng.gen_range(0..=100);
}
