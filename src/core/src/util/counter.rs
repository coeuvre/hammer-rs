use std::sync::{Arc, Mutex};
use std::num::{Zero, One};
use std::ops::Add;

pub struct Counter<T> {
    count: Arc<Mutex<T>>,
}

impl<T: Copy + Zero + One + Add<T, Output = T>> Counter<T> {
    pub fn new() -> Counter<T> {
        Counter {
            count: Arc::new(Mutex::new(T::zero())),
        }
    }

    pub fn next(&self) -> T {
        let mut count = self.count.lock().unwrap();
        let next = *count + T::one();
        *count = next;
        next
    }
}
