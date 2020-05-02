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
}

impl RefKindDesc {
    pub fn monitor_enter(&mut self) {
        //todo: impl
    }

    pub fn monitor_exit(&mut self) {
        //todo: impl
    }
}
