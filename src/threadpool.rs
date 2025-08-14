use std::thread::{self};
use std::usize;
use std::collections::VecDeque;
use std::sync::{Condvar, Mutex, Arc};

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
                closed: false
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

pub struct ThreadPool {
    pool: Vec<thread::JoinHandle<()>>,
    queue: Arc<UnboundedMpmcBlockingQueue>,
}

impl ThreadPool {
    pub fn new(n_threads: usize) -> Self {
        let mut pool = Vec::with_capacity(n_threads);
        let queue = Arc::new(UnboundedMpmcBlockingQueue::new());

        for _ in 0..n_threads {
            let q = queue.clone();
            let worker = thread::spawn(move || {
                loop {
                    let routine = q.get();
                    match routine {
                        None => break,
                        Some(job) => job(),
                    }
                }
            });

            pool.push(worker);  
        }

        return Self {
            pool: pool,
            queue: queue,
        }
    }

    pub fn submit<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static
    {
        self.queue.push(Box::new(job));
    }

    pub fn shutdown(&mut self) {
        self.queue.close();

        for worker in self.pool.drain(..) {
            worker.join().unwrap();
        }
    }
}