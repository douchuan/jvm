use crate::oop::{self, consts, Oop};
use crate::types::{FrameRef, JavaThreadRef};
use std::sync::{Arc, RwLock};

pub struct JavaThread {
    pub frames: Vec<FrameRef>,
    in_safe_point: bool,

    pub java_thread_obj: Option<Oop>,
    pub ex: Option<Oop>,
    pub is_alive: bool,
    pub eetop: i64,

    pub tag: String, //for debug
}

impl JavaThread {
    pub fn new(tag: Option<String>) -> JavaThreadRef {
        let eetop = gen_thread_id();
        let tag = tag.unwrap_or_else(|| format!("thread-{}", eetop));
        let t = Self {
            frames: Vec::new(),
            in_safe_point: false,

            java_thread_obj: None,
            ex: None,
            is_alive: false,
            eetop,
            tag,
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

fn gen_thread_id() -> i64 {
    use core::sync::atomic::Ordering;
    use std::sync::atomic::AtomicI64;
    static NEXT_ID: AtomicI64 = AtomicI64::new(0);
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    id
}
