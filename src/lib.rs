use std::num::NonZeroUsize;
use std::sync::{self, mpsc::Sender, Arc, Mutex};
use std::thread::JoinHandle;
use tracing::info_span;

use tracing::trace;

type Task = Box<dyn FnOnce() -> () + Send>;

pub struct ThreadPool {
    threads: Vec<JoinHandle<()>>,
    sender: Option<Sender<Task>>,
}

impl ThreadPool {
    pub fn new(size: NonZeroUsize) -> Self {
        let size = size.get();
        let mut threads = Vec::with_capacity(size);
        let (sender, receiver) = sync::mpsc::channel::<Task>();

        let receiver = Arc::new(Mutex::new(receiver));
        for _ in 0..size {
            let receiver = receiver.clone();
            let thread = std::thread::spawn(move || {
                let span = info_span!("worker", id = ?std::thread::current().id());
                let _enter = span.enter();
                trace!("Started",);
                loop {
                    let res = {
                        let rx = receiver.lock().unwrap();
                        rx.recv()
                    };

                    match res {
                        Ok(task) => {
                            trace!("Received task");
                            task();
                        }
                        Err(_) => {
                            trace!("Shutdown");
                            return;
                        }
                    }
                }
            });
            threads.push(thread);
        }

        Self {
            threads,
            sender: Some(sender),
        }
    }

    pub fn spawn<F>(&mut self, callback: F)
    where
        F: FnOnce() -> () + Send + 'static,
    {
        let Some(sender) = &mut self.sender else {
            panic!("ThreadPool sender must exist before drop");
        };
        sender.send(Box::new(callback)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        let sender = self.sender.take();
        drop(sender);

        while let Some(thread) = self.threads.pop() {
            thread.join().unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;
    use std::{num::NonZeroUsize, sync::atomic::AtomicUsize};

    use crate::ThreadPool;

    #[test]
    fn it_accepts_more_tasks_than_threads() {
        let mut pool = ThreadPool::new(NonZeroUsize::new(4).unwrap());
        let counter = Arc::new(AtomicUsize::new(0));
        let total = 10;

        for _ in 0..total {
            let counter = counter.clone();
            pool.spawn(move || {
                counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            });
        }

        for _ in 0..5 {
            let value = counter.load(std::sync::atomic::Ordering::SeqCst);
            if value == total {
                return;
            }
            std::thread::sleep(Duration::from_millis(10));
        }

        panic!("Counter never reached expected value");
    }
}
