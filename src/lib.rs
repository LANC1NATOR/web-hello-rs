use std::sync::{mpsc, Arc, Mutex};
use std::thread;

#[derive(Debug)]
pub enum PoolCreationError {
    InvalidSize(String),
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Builds a thread pool with the specified size.
    ///
    /// # Arguments
    ///
    /// * `size` - The number of threads to create in the pool.
    ///
    /// # Returns
    ///
    /// A `Result` containing the created `ThreadPool` if successful, or a `PoolCreationError` if the size is zero.
    pub fn build(size: usize) -> Result<ThreadPool, PoolCreationError> {
        match size {
            0 => Err(PoolCreationError::InvalidSize(
                "ThreadPool cannot be zero.".to_string(),
            )),
            _ => Ok({
                let (sender, receiver) = mpsc::channel();

                let receiver = Arc::new(Mutex::new(receiver));

                let mut workers = Vec::with_capacity(size);

                for id in 0..size {
                    workers.push(Worker::new(id, Arc::clone(&receiver)));
                }

                ThreadPool { workers, sender }
            }),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.send(job).unwrap();
    }
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv().unwrap();

            println!("Worker {id} got a job; executing.");
            job();
        });

        Worker { id, thread }
    }
}
