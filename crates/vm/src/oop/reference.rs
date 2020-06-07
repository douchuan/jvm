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
    pub fn monitor_enter(&self) {
        unsafe {
            self.mutex.lock();
        }
    }

    pub fn monitor_exit(&self) {
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

impl RefKind {
    pub fn extract_inst(&self) -> &InstOopDesc {
        match &self {
            RefKind::Inst(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn extract_array(&self) -> &ArrayOopDesc {
        match &self {
            RefKind::Array(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn extract_mut_array(&mut self) -> &mut ArrayOopDesc {
        let v = self;
        match v {
            RefKind::Array(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn extract_type_array(&self) -> &TypeArrayDesc {
        match &self {
            RefKind::TypeArray(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn extract_mut_type_array(&mut self) -> &mut TypeArrayDesc {
        let v = self;
        match v {
            RefKind::TypeArray(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn extract_mirror(&self) -> &MirrorOopDesc {
        match &self {
            RefKind::Mirror(v) => v,
            _ => unreachable!(),
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
