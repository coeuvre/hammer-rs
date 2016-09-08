use std::sync::{Arc, Mutex};
use std::iter::Step;

pub struct Counter<T> {
    count: Arc<Mutex<T>>,
}

impl<T: Copy + Step> Counter<T> {
    pub fn new(value: T) -> Counter<T> {
        Counter {
            count: Arc::new(Mutex::new(value)),
        }
    }

    pub fn next(&self) -> T {
        let mut count = self.count.lock().unwrap();
        let next = count.add_one();
        *count = next;
        next
    }
}
