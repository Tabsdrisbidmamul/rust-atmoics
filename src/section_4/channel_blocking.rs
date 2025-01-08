use std::{
    cell::UnsafeCell,
    marker::PhantomData,
    mem::MaybeUninit,
    sync::atomic::{AtomicBool, Ordering},
    thread::{self, Thread},
};

pub struct Channel<T> {
    message: UnsafeCell<MaybeUninit<T>>,
    ready: AtomicBool,
}

unsafe impl<T> Sync for Channel<T> where T: Send {}

pub struct Sender<'a, T> {
    channel: &'a Channel<T>,
    receiving_thread: Thread,
}

// Does not implement Send, stop Receiver being sent to other threads, so we know that Sender and Receiver are on the same thread (for this example, in reality we would use Sync to pass their references around and use that within each thread)
pub struct Receiver<'a, T> {
    channel: &'a Channel<T>,
    _no_send: PhantomData<*const ()>,
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
        (
            Sender {
                channel: self,
                receiving_thread: thread::current(),
            },
            Receiver {
                channel: self,
                _no_send: PhantomData,
            },
        )
    }
}

impl<T> Sender<'_, T> {
    pub fn send(self, message: T) {
        unsafe { (*self.channel.message.get()).write(message) };
        self.channel.ready.store(true, Ordering::Release);
        self.receiving_thread.unpark();
    }
}

impl<T> Receiver<'_, T> {
    #[allow(unused)]
    pub fn is_ready(&self) -> bool {
        self.channel.ready.load(Ordering::Relaxed)
    }

    ///
    /// Will park the thread until ready is true, park() and unpark() can return spuriously so by using memory ordering Acquire we can be sure that thread::park() and unpark() are placed in the correct order.
    ///
    pub fn receive(&self) -> T {
        while !self.channel.ready.swap(false, Ordering::Acquire) {
            thread::park();
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
 *
 * From section_4/channel_avoid_blocking.rs, we used pointers to remove Arc constraint. Now we have gone a set further and kept the reference of the current thread (the caller thread that calls split()) in sender to unpark() the thread and let receive to return back the value, if its ready else put the thread to sleep.
 *
 *
 */

pub fn channel_blocking_main() {
    let mut channel = Channel::<&str>::new();
    thread::scope(|s| {
        let (sender, receiver) = channel.split();
        s.spawn(move || {
            sender.send("hello world!");
        });

        let chan_msg = receiver.receive();
        println!("chan_msg {:?}", chan_msg);
        assert_eq!(chan_msg, "hello world!")
    });
}
