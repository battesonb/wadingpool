use rand::Rng;
use std::time::{Duration, Instant};
use tracing::{info, Level};

use wadingpool::ThreadPool;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .init();

    let mut pool = ThreadPool::new(std::thread::available_parallelism().unwrap());
    let (tx, rx) = std::sync::mpsc::channel();

    for _ in 0..20 {
        let tx = tx.clone();
        pool.spawn(move || {
            // Do some "work".
            let x = rand::thread_rng().gen_range(0..2500);
            std::thread::sleep(Duration::from_millis(x));

            let _ = tx.send(x);
        });
    }

    let mut acc = 0;
    let start = Instant::now();
    for _ in 0..5 {
        info!("Waiting...");
        std::thread::sleep(Duration::from_millis(500));
        while let Ok(res) = rx.try_recv() {
            acc += res;
            info!("Result: {res}");
        }
    }
    let elapsed = Instant::now().duration_since(start).as_millis();
    let rate = acc as u128 / elapsed;
    info!("Recevied {acc}ms of work in {elapsed}ms; a {rate}x improvement");
}
