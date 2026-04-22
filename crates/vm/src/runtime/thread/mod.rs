mod condvar;
mod java_thread;
mod main;
mod mutex;
mod thread_pool;
mod threads;

pub use condvar::Condvar;
pub use java_thread::current_java_thread;
pub use java_thread::JavaThread;
pub use java_thread::THREAD;
pub use main::MainThread;
pub use mutex::raw as mutex_raw;
pub use mutex::ReentrantMutex;
pub use thread_pool::ThreadPool;
pub use threads::Threads;

pub use java_thread::is_meet_ex;

/// Clear any pending exception on the current thread.
/// Used after best-effort class loading that may have failed.
pub fn clear_ex() {
    current_java_thread().write().unwrap().clear_ex();
}
