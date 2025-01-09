use std::{
    sync::atomic::{AtomicBool, Ordering::SeqCst},
    thread,
};

// check `seqcst.md` for notes

static A: AtomicBool = AtomicBool::new(false);
static B: AtomicBool = AtomicBool::new(false);

static mut S: String = String::new();

pub fn seq_cst_ordering() {
    let a = thread::spawn(|| {
        A.store(true, SeqCst);
        if !B.load(SeqCst) {
            unsafe {
                #[allow(static_mut_refs)]
                S.push('!');
            }
        }
    });

    let b = thread::spawn(|| {
        B.store(true, SeqCst);
        if !A.load(SeqCst) {
            unsafe {
                #[allow(static_mut_refs)]
                S.push('!');
            }
        }
    });

    a.join().unwrap();
    b.join().unwrap();

    println!("S is {:?}", unsafe {
        #[allow(static_mut_refs)]
        &S
    });
}
