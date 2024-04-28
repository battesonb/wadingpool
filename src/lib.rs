use std::num::NonZeroUsize;
use std::thread::JoinHandle;

pub struct ThreadPool {
    threads: Vec<JoinHandle<()>>,
}

impl ThreadPool {
    pub fn new(size: NonZeroUsize) -> Self {
        let size = size.get();
        let mut threads = Vec::with_capacity(size);

        for _ in 0..size {
            let thread = std::thread::spawn(move || {
                std::thread::park();
            });
            threads.push(thread);
        }

        Self { threads }
    }
}
