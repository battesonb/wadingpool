use std::num::NonZeroUsize;
use std::sync::{self, mpsc::Sender, Arc, Mutex};
use std::thread::JoinHandle;

type Task = Box<dyn FnOnce() -> () + Send>;

pub struct ThreadPool {
    threads: Vec<JoinHandle<()>>,
    sender: Sender<Task>,
}

impl ThreadPool {
    pub fn new(size: NonZeroUsize) -> Self {
        let size = size.get();
        let mut threads = Vec::with_capacity(size);
        let (sender, receiver) = sync::mpsc::channel::<Task>();

        let receiver = Arc::new(Mutex::new(receiver));
        for _ in 0..size {
            let receiver = receiver.clone();
            let thread = std::thread::spawn(move || loop {
                if let Ok(task) = {
                    let rx = receiver.lock().unwrap();
                    rx.recv()
                } {
                    task();
                }
            });
            threads.push(thread);
        }

        Self { threads, sender }
    }

    pub fn spawn<F>(&mut self, callback: F)
    where
        F: FnOnce() -> () + Send + 'static,
    {
        self.sender.send(Box::new(callback)).unwrap();
    }
}
