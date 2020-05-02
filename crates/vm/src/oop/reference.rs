use crate::oop::{ArrayOopDesc, InstOopDesc, MirrorOopDesc, TypeArrayDesc};
use crate::runtime::ReentrantMutex;
use std::fmt;
use std::fmt::Formatter;
use std::sync::{Condvar, Mutex};

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
}

impl RefKindDesc {
    pub fn new(v: RefKind) -> Self {
        let mutex = unsafe {
            let mut mutex = ReentrantMutex::uninitialized();
            mutex.init();
            mutex
        };

        Self {
            v,
            hash_code: None,
            mutex,
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
