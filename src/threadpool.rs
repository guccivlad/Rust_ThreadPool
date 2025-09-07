use crate::unbounded_mpmc_blocking_queue::UnboundedMpmcBlockingQueue;
use std::cell::RefCell;
use std::sync::Arc;
use std::thread::{self};
use std::usize;

pub struct Inner {
    queue: Arc<UnboundedMpmcBlockingQueue>,
}

impl Inner {
    pub fn submit<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.queue.push(Box::new(job));
    }
}

pub struct ThreadPool {
    pool: Vec<thread::JoinHandle<()>>,
    inner: Inner,
}

thread_local! {
    static CURRENT_THREAD_POOL: RefCell<Option<Arc<UnboundedMpmcBlockingQueue>>> = RefCell::new(None);
}

impl ThreadPool {
    pub fn new(n_threads: usize) -> Self {
        let mut pool = Vec::with_capacity(n_threads);
        let inner = Inner {
            queue: Arc::new(UnboundedMpmcBlockingQueue::new()),
        };

        for _ in 0..n_threads {
            let q = inner.queue.clone();
            let worker = thread::spawn(move || {
                CURRENT_THREAD_POOL.with(|slot| *slot.borrow_mut() = Some(q.clone()));
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

        return ThreadPool {
            pool: pool,
            inner: inner,
        };
    }

    pub fn current() -> Option<Inner> {
        CURRENT_THREAD_POOL.with(|slot|
            slot.borrow().as_ref().map(|arc| Inner{queue: arc.clone()})
        )
    }

    pub fn submit<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.inner.queue.push(Box::new(job));
    }

    pub fn shutdown(&mut self) {
        self.inner.queue.close();

        for worker in self.pool.drain(..) {
            worker.join().unwrap();
        }
    }
}
