use std::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};

// Private impl, pub fn exists to return tuple pair of Sender, Receiver
struct Channel<T> {
    message: UnsafeCell<MaybeUninit<T>>,
    ready: AtomicBool,
}

pub struct Sender<T> {
    channel: Arc<Channel<T>>,
}

pub struct Receiver<T> {
    channel: Arc<Channel<T>>,
}

unsafe impl<T> Sync for Channel<T> where T: Send {}

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let arc = Arc::new(Channel {
        message: UnsafeCell::new(MaybeUninit::<T>::uninit()),
        ready: AtomicBool::new(false),
    });
    (
        Sender {
            channel: arc.clone(),
        },
        Receiver { channel: arc },
    )
}

impl<T> Sender<T> {
    // the send method takes ownership of Sender and will be dropped when this function exits. This is a one-off sender, so another .send() will throw an error
    pub fn send(self, message: T) {
        unsafe { (*self.channel.message.get()).write(message) };
        self.channel.ready.store(true, Ordering::Release);
    }
}

impl<T> Receiver<T> {
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

pub fn channel_send_receive() {
    thread::scope(|s| {
        let (sender, receiver) = channel::<&str>();
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
