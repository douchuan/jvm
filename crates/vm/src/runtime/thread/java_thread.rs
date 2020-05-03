use crate::oop::{self, consts, Oop};
use crate::types::{FrameRef, JavaThreadRef};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

lazy_static! {
    static ref NATIVE_THREAD_POOL: Mutex<HashMap<std::thread::ThreadId, JavaThreadRef>> =
        { Mutex::new(HashMap::new()) };
}

pub struct JavaThread {
    pub frames: Vec<FrameRef>,
    in_safe_point: bool,

    pub java_thread_obj: Option<Oop>,
    pub ex: Option<Oop>,
}

impl JavaThread {
    pub fn new() -> JavaThreadRef {
        let t = Self {
            frames: Vec::new(),
            in_safe_point: false,

            java_thread_obj: None,
            ex: None,
        };
        Arc::new(RwLock::new(Box::new(t)))
    }

    pub fn set_java_thread_obj(&mut self, obj: Oop) {
        self.java_thread_obj = Some(obj);
    }
}

//exception
impl JavaThread {
    pub fn set_ex(&mut self, ex: Oop) {
        self.ex = Some(ex);
    }

    pub fn is_meet_ex(&self) -> bool {
        self.ex.is_some()
    }

    pub fn take_ex(&mut self) -> Option<Oop> {
        self.ex.take()
    }
}


