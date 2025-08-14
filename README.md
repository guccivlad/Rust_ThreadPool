# Educational Thread Pool and WaitGroup (Rust)

A small learning project: a simple **thread pool** and a **WaitGroup** (Go-style) written in safe Rust. Great for experimenting with basic synchronization primitives, task queues, and coordinated shutdown.

> ⚠️ Educational project: the code favors clarity over peak performance and production-grade resilience.

---

## Features
- Fixed-size worker pool (`ThreadPool::new(n)`)
- Submit tasks as `FnOnce() + Send + 'static`
- **`WaitGroup`** primitive: `add(delta)`, `done()`, `wait()`

---

## Quick start
```bash
cargo run
```

Minimal usage examples:

```rust
fn main() {
    let first_count = Arc::new(Mutex::new(0));
    let second_count = Arc::new(Mutex::new(0));
    
    let mut pool = ThreadPool::new(4);
    let wg = WaitGroup::new();

    for _ in 0..256 {
        wg.add(1);

        let wg_clone = wg.clone();
        let count = Arc::clone(&first_count);
        pool.submit(move || {
            {
                *count.lock().unwrap() += 1;
            }
            wg_clone.done();
        });
    }

    wg.wait();

    for _ in 0..10_000_000 {
        wg.add(1);

        let wg_clone = wg.clone();
        let count = Arc::clone(&second_count);
        pool.submit(move || {
            {
                *count.lock().unwrap() += 1;
            }
            wg_clone.done();
        });
    }

    pool.shutdown();

    println!("First result: {}", *first_count.lock().unwrap());
    println!("Second result: {}", *second_count.lock().unwrap());
}
```

```rust
fn main() {
    let pool = ThreadPool::new(4);
    let wg = WaitGroup::new();

    for i in 0..8_000 {
        wg.add(1);
        let wg2 = wg.clone();
        pool.submit(move || {
            do_work(i);
            wg2.done();
        });
    }

    wg.wait();
    println!("All tasks are completed");
}

fn do_work(i: usize) {
    std::thread::sleep(std::time::Duration::from_millis(50));
    println!("job #{i} done");
}
```

### API

```ThreadPool```

```rust
impl ThreadPool {
    pub fn new(size: usize) -> Self;

    pub fn submit<F>(&self, job: F)
    where F: FnOnce() + Send + 'static;

    pub fn shutdown(&mut self);
}
```

$\textbf{Design idea:}$ a task queue(```UnboundedMpmcBlockingQueue```) plus a set of worker threads. Each worker pulls a task from the queue and executes it.

```UnboundedMpmcBlockingQueue```

```rust
impl UnboundedMpmcBlockingQueue {
    // Create new Queue
    pub fn new() -> Self;

    // Wait and remove the item from the head of the queue; if the queue is closed and empty, then return None
    pub fn get(&self) -> Option<Job>;

    // Add an item to the back of the queue
    pub fn push(&self, job: Job);

    // Close the queue for new Push messages. Items that have already been added to the queue will remain available for retrieval
    pub fn close(&self);
}
```

```WaitGroup```

```rust
impl WaitGroup {
    pub fn new() -> Self;

    pub fn add(&self, delta: i32);

    pub fn done(&self);

    pub fn wait(&self);
}
```

$\textbf{Design idea:}$ a shared counter and a Condvar to wait.
