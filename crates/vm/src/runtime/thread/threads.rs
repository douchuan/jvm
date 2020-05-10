use crate::runtime;
use crate::runtime::thread::ThreadPool;
use crate::types::JavaThreadRef;
use std::borrow::Borrow;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::{Arc, Condvar, Mutex};

pub struct Threads {
    pool: Mutex<ThreadPool>,
    threads: Mutex<Vec<JavaThreadRef>>,
    cond_join: Condvar,
    next_id: AtomicI64,
}

impl Threads {
    pub fn new(thread_pool_count: usize) -> Threads {
        Threads {
            pool: Mutex::new(ThreadPool::new(thread_pool_count)),
            threads: Mutex::new(Vec::new()),
            cond_join: Condvar::new(),
            next_id: AtomicI64::new(1),
        }
    }
}

impl Threads {
    pub fn next_id(&self) -> i64 {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }

    pub fn attach_current_thread(&self) {
        runtime::thread::THREAD.with(|thread| {
            let mut threads = self.threads.lock().unwrap();
            threads.push(thread.borrow().clone());
        });
    }

    pub fn attach_java_thread(&self, thread: JavaThreadRef) {
        let mut threads = self.threads.lock().unwrap();
        threads.push(thread);
    }

    pub fn detach_current_thread(&self) {
        runtime::thread::THREAD.with(|thread| {
            let mut threads = self.threads.lock().unwrap();
            threads.retain(|elem| !Arc::ptr_eq(elem, &*thread.borrow()));
            self.cond_join.notify_all();
        });
    }

    pub fn find_java_thread(&self, eetop: i64) -> Option<JavaThreadRef> {
        let mut threads = self.threads.lock().unwrap();
        threads
            .iter()
            .find(|t| t.read().unwrap().eetop == eetop)
            .map(|t| t.clone())
    }

    pub fn join_all(&self) {
        let mut threads = self.threads.lock().unwrap();

        while threads.len() > 0 {
            threads = self.cond_join.wait(threads).unwrap();
        }
    }

    pub fn spawn_java_thread<F: FnOnce() + Send + 'static>(&self, f: F) {
        let pool = self.pool.lock().unwrap();
        pool.execute(f);
    }
}
