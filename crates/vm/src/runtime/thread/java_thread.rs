use crate::oop::{self, consts, Oop};
use crate::types::{FrameRef, JavaThreadRef};
use std::cell::RefCell;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};

thread_local! {
    pub static THREAD: RefCell<JavaThreadRef> = RefCell::new(JavaThread::main());
    pub static IS_MEET_EX: AtomicBool = AtomicBool::new(false);
}

pub fn current_java_thread() -> JavaThreadRef {
    THREAD.with(|t| t.borrow().clone())
}

#[inline]
pub fn is_meet_ex() -> bool {
    IS_MEET_EX.with(|v| v.load(Ordering::Relaxed))
}

#[inline]
fn set_meet_ex(val: bool) {
   IS_MEET_EX.with(|v| v.store(val, Ordering::Relaxed));
}

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
    pub fn new(tag: Option<String>, eetop: i64) -> JavaThreadRef {
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

    pub fn main() -> JavaThreadRef {
        JavaThread::new(Some("main".to_string()), 0)
    }

    pub fn set_java_thread_obj(&mut self, obj: Oop) {
        self.java_thread_obj = Some(obj);
    }
}

//exception
impl JavaThread {
    pub fn set_ex(&mut self, ex: Oop) {
        set_meet_ex(true);
        self.ex = Some(ex);
    }

    pub fn take_ex(&mut self) -> Option<Oop> {
        set_meet_ex(false);
        self.ex.take()
    }
}
