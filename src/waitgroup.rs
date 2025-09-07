use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::{AcqRel, Acquire, Release};
use std::sync::Arc;
use crate::waitwake;

pub struct WaitGroup {
    state: Arc<AtomicU32>,
    generation: Arc<AtomicU32>,
}

impl WaitGroup {
    pub fn new() -> Self {
        return Self {
            state: Arc::new(AtomicU32::new(0)),
            generation: Arc::new(AtomicU32::new(0)),
        }
    }

    pub fn add(&self, delta: i32) {
        if delta > 0 {
            self.state.fetch_add(delta as u32, AcqRel);
        } else if delta < 0 {
            let k = (-delta) as u32;
            let prev = self.state.fetch_sub(k, AcqRel);

            if prev < k {
                panic!("Negative WiatGroup count");
            }

            if prev == k {
                self.generation.fetch_add(1, Release);
                waitwake::wake_all(&self.generation);
            }
        } else {}
    }

    pub fn done(&self) {
        self.add(-1);
    }

    pub fn wait(&self) {
        loop {
            let g = self.generation.load(Acquire);

            if self.state.load(Acquire) == 0 { return; }

            waitwake::wait(&self.generation, g);
        }
    }
}

impl Clone for WaitGroup {
    fn clone(&self) -> Self {
        return Self {
            state: Arc::clone(&self.state),
            generation: Arc::clone(&self.generation),
        };
    }
}
