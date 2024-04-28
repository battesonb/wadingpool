use std::time::{Duration, Instant};

use wadingpool::ThreadPool;

const COMPUTE_MILLIS: u64 = 50_000;

fn main() {
    let tasks_ms = vec![2, 5, 10, 20, 50, 100, 250, 500, 1000];

    println!("task_ms,elapsed_ms");
    for task_ms in tasks_ms {
        let total_tasks = COMPUTE_MILLIS / task_ms;
        let mut pool = ThreadPool::new(std::thread::available_parallelism().unwrap());
        let (tx, rx) = std::sync::mpsc::channel();
        for _ in 0..total_tasks {
            let tx = tx.clone();
            pool.spawn(move || {
                std::thread::sleep(Duration::from_millis(task_ms));
                let _ = tx.send(());
            });
        }

        let start = Instant::now();
        for _ in 0..total_tasks {
            let _ = rx.recv().unwrap();
        }
        let elapsed = Instant::now().duration_since(start).as_millis();
        println!("{task_ms},{elapsed}");
    }
}
