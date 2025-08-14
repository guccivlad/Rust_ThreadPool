use std::sync::{Arc, Condvar, Mutex};

pub struct WaitGroup {
    counter: Arc<Mutex<i32>>,
    is_running: Arc<Condvar>,
}

impl WaitGroup {
    pub fn new() -> Self {
        return Self {
            counter: Arc::new(Mutex::new(0)),
            is_running: Arc::new(Condvar::new()),
        };
    }

    pub fn add(&self, delta: i32) {
        let mut count = self.counter.lock().unwrap();
        *count += delta;

        if *count == 0 {
            self.is_running.notify_all();
        }
    }

    pub fn done(&self) {
        self.add(-1);
    }

    pub fn wait(&self) {
        let mut count = self.counter.lock().unwrap();
        while *count > 0 {
            count = self.is_running.wait(count).unwrap();
        }
    }
}

impl Clone for WaitGroup{
    fn clone(&self) -> Self {
        return Self {
            counter: Arc::clone(&self.counter),
            is_running: Arc::clone(&self.is_running),
        }
    }
}