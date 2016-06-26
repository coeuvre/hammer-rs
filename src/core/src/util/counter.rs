use std::sync::{Arc, Mutex};

pub struct Counter {
    count: Arc<Mutex<u64>>,
}

impl Counter {
    pub fn new() -> Counter {
        Counter {
            count: Arc::new(Mutex::new(0)),
        }
    }

    pub fn next(&self) -> u64 {
        let mut count = self.count.lock().unwrap();
        let next = *count + 1;
        *count = next;
        next
    }
}
