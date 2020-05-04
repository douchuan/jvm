use crate::types::JavaThreadRef;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

lazy_static! {
    //fixme: the count of pool threads should be configured
    static ref THREAD_POOL: Mutex<ThreadPool> = { Mutex::new(ThreadPool::new(5)) };
    static ref THREAD_REGISTRY: Mutex<HashMap<std::thread::ThreadId, JavaThreadRef>> =
        { Mutex::new(HashMap::new()) };
    static ref MAIN_MUTEX: Mutex<usize> = { Mutex::new(0) };
    static ref MAIN_COND: Condvar = { Condvar::new() };
}

pub fn init() {
    lazy_static::initialize(&THREAD_POOL);
    lazy_static::initialize(&THREAD_REGISTRY);
    lazy_static::initialize(&MAIN_MUTEX);
    lazy_static::initialize(&MAIN_COND);
}

pub fn spawn_java_thread<F: FnOnce() + Send + 'static>(f: F) {
    let pool = THREAD_POOL.lock().unwrap();
    pool.execute(f);
}

//called in some thread context
pub fn register_jt(jt: JavaThreadRef) {
    let tid = std::thread::current().id();
    let mut reg = THREAD_REGISTRY.lock().unwrap();
    reg.insert(tid, jt);
}

pub fn un_register_jt() {
    let tid = std::thread::current().id();
    let mut reg = THREAD_REGISTRY.lock().unwrap();
    reg.remove(&tid);
}

pub fn obtain_jt(eetop: i64) -> Option<JavaThreadRef> {
    let reg = THREAD_REGISTRY.lock().unwrap();
    for v in reg.values() {
        if v.read().unwrap().eetop == eetop {
            return Some(v.clone());
        }
    }
    None
}

pub fn park_if_needed() {
    loop {
        let n = {
            let reg = THREAD_REGISTRY.lock().unwrap();
            reg.len()
        };

        if n > 0 {
            let handle = MAIN_MUTEX.lock().unwrap();
            MAIN_COND.wait(handle).unwrap();
        } else {
            break;
        }
    }
}

pub fn wake_up_main() {
    MAIN_COND.notify_one();
}

pub fn obtain_current_jt() -> Option<JavaThreadRef> {
    let tid = std::thread::current().id();
    let reg = THREAD_REGISTRY.lock().unwrap();
    reg.get(&tid).map(|it| it.clone())
}

enum Message {
    NewJob(Job),
    Terminate,
}

struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

trait FnBox {
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<F>) {
        (*self)()
    }
}

type Job = Box<FnBox + Send + 'static>;

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        info!("Sending terminate message to all workers.");

        for _ in &mut self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        info!("Shutting down all workers.");

        for worker in &mut self.workers {
            info!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();

            match message {
                Message::NewJob(job) => {
                    warn!("Worker-{} got a job, executing", id);
                    job.call_box();
                    warn!("Worker-{}, fin", id);
                }
                Message::Terminate => {
                    warn!("Worker {} was told to terminate.", id);
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}
