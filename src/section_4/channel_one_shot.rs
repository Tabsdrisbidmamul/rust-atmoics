use std::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::atomic::{AtomicBool, Ordering},
    thread,
};

pub struct Channel<T> {
    message: UnsafeCell<MaybeUninit<T>>,
    in_use: AtomicBool, // once set it actually is never reset (hence why its a one-off channel) p.93-94 uses an AtomicU8, and uses an enum to create a finite state machine which will stop this error
    ready: AtomicBool,
}

unsafe impl<T> Sync for Channel<T> where T: Send {}

impl<T> Channel<T> {
    pub const fn new() -> Self {
        Self {
            message: UnsafeCell::new(MaybeUninit::uninit()),
            in_use: AtomicBool::new(false),
            ready: AtomicBool::new(false),
        }
    }

    // Safety: Only call once, if a thread calls it multiple times, may overwrite the previously stored message, causing data races
    pub fn send(&self, message: T) {
        if self.in_use.swap(true, Ordering::Relaxed) {
            panic!("can't send more than one message");
        }

        unsafe {
            (*self.message.get()).write(message);
        }
        self.ready.store(true, Ordering::Release);
    }

    ///
    /// Send and Receive use Release and Acquire memory loading, is_ready() is fine to use Relaxed
    ///
    pub fn is_ready(&self) -> bool {
        self.ready.load(Ordering::Relaxed)
    }

    ///
    /// Panics if no message is available (useful if called multiple times from n threads when they don't check for is_ready()), use is_ready() to check if a message is available.
    ///
    /// ---- No longer creates an unsafe copy ----
    /// Safety: Only call once, if a thread calls it multiple times, an unsafe copy of T will be given to n threads who call receive after the first thread has called it.
    ///
    /// Even though T does not implement Copy, an unsafe Copy is made, which is not intended behaviour
    ///
    pub fn receive(&self) -> T {
        if !self.ready.swap(false, Ordering::Acquire) {
            panic!("no message available!");
        }

        // Safety: we've checked and reset the ready flag
        unsafe { (*self.message.get()).assume_init_read() }
    }
}

impl<T> Drop for Channel<T> {
    fn drop(&mut self) {
        if *self.ready.get_mut() {
            unsafe { self.message.get_mut().assume_init_drop() }
        }
    }
}

pub fn channel_one_off_main() {
    let channel = Channel::<&str>::new();
    let t = thread::current();
    thread::scope(|s| {
        s.spawn(|| {
            channel.send("hello world!");
            t.unpark();
        });

        while !channel.is_ready() {
            thread::park();
        }

        let chan_msg = channel.receive();
        println!("chan_msg {:?}", chan_msg);
        assert_eq!(chan_msg, "hello world!");
    });
}
