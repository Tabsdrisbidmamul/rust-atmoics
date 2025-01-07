use std::{
    collections::VecDeque,
    sync::{Condvar, Mutex},
};

pub struct Channel<T> {
    queue: Mutex<VecDeque<T>>,
    item_ready: Condvar,
}
