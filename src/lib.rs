use std::num::NonZeroUsize;
use std::sync::{self, mpsc::Sender, Arc, Mutex};
use std::thread::JoinHandle;

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
            let thread = std::thread::spawn(move || loop {
                let res = {
                    let rx = receiver.lock().unwrap();
                    rx.recv()
                };

                match res {
                    Ok(task) => {
                        task();
                    }
                    Err(_) => {
                        return;
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
