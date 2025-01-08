use std::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::atomic::{AtomicBool, Ordering},
    thread,
};

pub struct Channel<T> {
    message: UnsafeCell<MaybeUninit<T>>,
    ready: AtomicBool,
}

unsafe impl<T> Sync for Channel<T> where T: Send {}

pub struct Sender<'a, T> {
    channel: &'a Channel<T>,
}

pub struct Receiver<'a, T> {
    channel: &'a Channel<T>,
}

impl<T> Channel<T> {
    pub const fn new() -> Self {
        Self {
            message: UnsafeCell::new(MaybeUninit::<T>::uninit()),
            ready: AtomicBool::new(false),
        }
    }

    ///
    /// Will take a mut reference, initialise a new Channel object at the pointer's address and return a tuple of Sender and Receiver with the initialised value within the pointer.
    ///
    /// By derefing, we ensure previous object initialisations are dropped (Rust handles it if it does exist)
    ///
    pub fn split<'a>(&'a mut self) -> (Sender<'a, T>, Receiver<'a, T>) {
        *self = Self::new();
        (Sender { channel: self }, Receiver { channel: self })
    }
}

impl<T> Sender<'_, T> {
    pub fn send(self, message: T) {
        unsafe { (*self.channel.message.get()).write(message) };
        self.channel.ready.store(true, Ordering::Release);
    }
}

impl<T> Receiver<'_, T> {
    pub fn is_ready(&self) -> bool {
        self.channel.ready.load(Ordering::Relaxed)
    }

    pub fn receive(&self) -> T {
        if !self.channel.ready.swap(false, Ordering::Acquire) {
            panic!("no message available!");
        }

        unsafe { (*self.channel.message.get()).assume_init_read() }
    }
}

impl<T> Drop for Channel<T> {
    fn drop(&mut self) {
        if *self.ready.get_mut() {
            unsafe { self.message.get_mut().assume_init_drop() }
        }
    }
}

/**
 * From section_4/channel_sender_receiver.rs, we used an Arc to keep track of a shared channel between Sender and Receiver.
 *
 * Here we use mut refs instead, within the split method we take in the mut ref and deref it to access the value at the pointer address to initialise a new Channel object within it. This will drop any previous Channel and ensure multiple calls to split() are deallocated gracefully.
 *
 * The rest of the implementation is the same as section_4/channel_sender_receiver. Only that we no longer have the overhead of an Arc, just using pointers so more manual control from us to make sure we do not segfault.
 */

pub fn channel_avoid_borrowing_main() {
    let mut channel = Channel::<&str>::new();
    thread::scope(|s| {
        let (sender, receiver) = channel.split();
        let t = thread::current();
        s.spawn(move || {
            sender.send("hello world!");
            t.unpark();
        });

        while !receiver.is_ready() {
            thread::park();
        }

        let chan_msg = receiver.receive();
        println!("chan_msg {:?}", chan_msg);
        assert_eq!(chan_msg, "hello world!")
    });
}
