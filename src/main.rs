use test_project::threadpool::ThreadPool;
use std::sync::{Arc, Mutex};
use test_project::waitgroup::WaitGroup;

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