use crate::oop::{ArrayOopDesc, InstOopDesc, MirrorOopDesc, TypeArrayDesc};
use std::sync::{Condvar, Mutex};

#[derive(Debug)]
pub enum RefKind {
    Inst(InstOopDesc),
    Array(ArrayOopDesc),
    TypeArray(TypeArrayDesc),
    Mirror(MirrorOopDesc),
}

#[derive(Debug)]
pub struct RefKindDesc {
    pub v: RefKind,
    pub hash_code: Option<i32>,

    // Do these two fields make sense? The operation itself, implicit lock
    pub cond: Condvar,
    pub monitor: Mutex<usize>,
}

impl RefKindDesc {
    pub fn monitor_enter(&mut self) {
        let mut v = self.monitor.lock().unwrap();
        *v += 1;
    }

    pub fn monitor_exit(&mut self) {
        let mut v = self.monitor.lock().unwrap();
        *v -= 1;
    }
}
