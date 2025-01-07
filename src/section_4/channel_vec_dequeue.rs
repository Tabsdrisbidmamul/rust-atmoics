use std::{
    collections::VecDeque,
    sync::{Condvar, Mutex},
};

#[allow(unused)]
pub struct Channel<T> {
    queue: Mutex<VecDeque<T>>,
    item_ready: Condvar,
}

/**
 * Channels allow data to be sent from one thread to another or n threads. Pretty much the pub/sub model in bus services.
 *
 * We ues a Mutex to wrap a VecDeque (efficient at adding/ removing at the front/back), and use a CondVar to notify one to n threads of a message.
 *
 * Send will place the message in the queue, and receive will consume the message or put the thread to sleep, and will wake up if the CondVar notifies it to wake up and consume a message.
 *
 * problems with this basic implementation is that
 * - The queue can grow, so whilst one thread is handling the reallocation, other threads are being blocked from sending/ receiving.
 * - Threads are blocked by the Mutex when they want to send/ receive from the queue.
 *
 * Rust provides the mpsc (multiple producers single consumer) channel type in the std lib. That overcomes our simple Mutex and VecDequeue example
 *
 * crossbeam-channel crate allows multiple consumers.
 *
 *
 *
 */

impl<T> Channel<T> {
    #[allow(unused)]
    pub fn new(&self) -> Self {
        Self {
            queue: Mutex::new(VecDeque::<T>::new()),
            item_ready: Condvar::new(),
        }
    }

    #[allow(unused)]
    pub fn send(&self, message: T) {
        self.queue.lock().unwrap().push_back(message);
        self.item_ready.notify_one();
    }

    #[allow(unused)]
    pub fn receive(&self) -> T {
        let mut guard = self.queue.lock().unwrap();
        loop {
            if let Some(message) = guard.pop_front() {
                return message;
            }

            // block thread in loop till there is a message in the queue to return
            guard = self.item_ready.wait(guard).unwrap();
        }
    }
}
