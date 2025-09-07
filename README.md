# Educational Thread Pool and WaitGroup (Rust)

A small learning project: a simple **thread pool** and a [**WaitGroup**](https://pkg.go.dev/sync#WaitGroup)  written in Rust. Great for experimenting with basic synchronization primitives, task queues, and coordinated shutdown.

> ⚠️ Educational project: the code favors clarity over peak performance and production-grade resilience.

---

## Thread Pool API
`new(n_threads: usize) -> ThreadPool`

Creates a new thread pool with the number of `n_threads` workers

`submit<F>(&self, job: F) where F: FnOnce() + Send + 'static`

Planning a task for execution

`shutdown(&mut self)`

Stopping the thread pool. The `shutdown` call returns control when all pool threads are stopped.

`current() -> Option<Inner>`

An executable task can get a pointer to the task queue of the current pool using the static `current` method.

In a good way, you need to return a link to the current thread pool, but I do not know how to do this🙂. If you know how to do this, I would appreciate a hint.

## Unbounded Mpmc Blocking Queue API

`get(&self) -> Option<Job>`

Wait and extract an item from the head of the queue

`push(&self, job: Job)`

Add an item to the back of the queue. If the queue is closed, there will be a panic

`close(&self)`

Close the queue

## Wait Group API

`add(&self, delta: i32)`

Add a positive delta

`done(&self)`

Subtract one

`wait(&self)`

Block the current stream until the counter value drops to zero.

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