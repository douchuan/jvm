use crate::oop::{ArrayOopDesc, InstOopDesc, MirrorOopDesc, TypeArrayDesc};
use crate::runtime::thread::{Condvar, ReentrantMutex};
use std::fmt;
use std::fmt::Formatter;
use std::time::Duration;

#[derive(Debug)]
pub enum RefKind {
    Inst(InstOopDesc),
    Array(ArrayOopDesc),
    TypeArray(TypeArrayDesc),
    Mirror(MirrorOopDesc),
}

pub struct RefKindDesc {
    pub v: RefKind,
    pub hash_code: Option<i32>,

    mutex: ReentrantMutex,
    cond_var: Condvar,
}

impl RefKindDesc {
    pub fn new(v: RefKind) -> Self {
        let mutex = unsafe {
            let mut mutex = ReentrantMutex::uninitialized();
            mutex.init();
            mutex
        };

        let cond_var = unsafe {
            let mut cond = Condvar::new();
            cond.init();
            cond
        };

        Self {
            v,
            hash_code: None,
            mutex,
            cond_var,
        }
    }
}

impl RefKindDesc {
    pub fn monitor_enter(&mut self) {
        unsafe {
            self.mutex.lock();
        }
    }

    pub fn monitor_exit(&mut self) {
        unsafe {
            self.mutex.unlock();
        }
    }

    pub fn wait(&self) {
        unsafe {
            self.cond_var.wait(&self.mutex);
        }
    }

    pub fn wait_timeout(&self, duration: Duration) {
        unsafe {
            self.cond_var.wait_timeout(&self.mutex, duration);
        }
    }

    pub fn notify_all(&self) {
        unsafe {
            self.cond_var.notify_all();
        }
    }
}

impl Drop for RefKindDesc {
    fn drop(&mut self) {
        unsafe {
            self.mutex.destroy();
        }
    }
}

impl fmt::Debug for RefKindDesc {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("RefKindDesc")
            .field("v", &self.v)
            .field("hash_code", &self.hash_code)
            .field("mutex", &"mutex")
            .finish()
    }
}
