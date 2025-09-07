use std::collections::VecDeque;
use std::sync::{Condvar, Mutex};

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Inner {
    queue: VecDeque<Job>,
    closed: bool,
}

pub struct UnboundedMpmcBlockingQueue {
    inner: Mutex<Inner>,
    not_empty: Condvar,
}

impl UnboundedMpmcBlockingQueue {
    pub fn new() -> Self {
        return Self {
            inner: Mutex::new(Inner {
                queue: VecDeque::new(),
                closed: false,
            }),
            not_empty: Condvar::new(),
        };
    }

    pub fn get(&self) -> Option<Job> {
        let mut inner_ = self.inner.lock().unwrap();
        while inner_.queue.is_empty() && !inner_.closed {
            inner_ = self.not_empty.wait(inner_).unwrap();
        }

        let routine = inner_.queue.pop_back();
        match routine {
            None => return None,
            Some(job) => return Some(job),
        }
    }

    pub fn push(&self, job: Job) {
        let mut inner_ = self.inner.lock().unwrap();
        if inner_.closed {
            panic!("UnboundedMpmcBlockingQueue closed!");
        }
        inner_.queue.push_front(job);
        self.not_empty.notify_one();
    }

    pub fn close(&self) {
        let mut inner_ = self.inner.lock().unwrap();
        if inner_.closed {
            panic!("UnboundedMpmcBlockingQueue closed!");
        }
        inner_.closed = true;
        self.not_empty.notify_all();
    }
}
