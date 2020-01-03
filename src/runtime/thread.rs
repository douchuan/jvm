use crate::oop::{InstOopDesc, Method, MethodId, OopDesc};
use crate::runtime::{self, Frame};
use std::borrow::BorrowMut;
use std::sync::Arc;

pub struct JavaThread {
    frames: Vec<Frame>,
    args: Option<Vec<Arc<OopDesc>>>,
    pc: u32,
    in_safe_point: bool,

    java_thread_obj: Option<Arc<OopDesc>>,
    pub exception: Option<Arc<OopDesc>>,

    method: Method,
}

pub struct JavaMainThread {
    pub main_class: String,
    pub args: Option<Vec<String>>,
}

impl JavaThread {
    pub fn new(method: &MethodId, args: Option<Vec<Arc<OopDesc>>>) -> Self {
        Self {
            frames: Vec::new(),
            args,
            pc: 0,
            in_safe_point: false,

            java_thread_obj: None,
            exception: None,

            method: method.method.clone(),
        }
    }

    pub fn run(this: Arc<JavaThread>) {
        //todo: impl
    }

    pub fn throw_ext(mut this: Arc<JavaThread>, ext: &[u8], rethrow: bool) {
        //todo: impl
    }

    pub fn throw_ext_with_msg(mut this: Arc<JavaThread>, ext: &[u8], rethrow: bool, msg: String) {
        //todo: impl
    }

    pub fn throw_ext_with_msg2(mut this: Arc<JavaThread>, ext: &[u8], rethrow: bool, msg: &[u8]) {
        //todo: impl
    }

    pub fn try_handle_exception(mut this: Arc<JavaThread>, ex: Arc<OopDesc>) -> i32 {
        //todo: impl
        unimplemented!()
    }

    pub fn clear_ext(mut this: Arc<JavaThread>) {
        let this = Arc::get_mut(&mut this).unwrap();
        this.exception = None;
    }

    pub fn is_exception_occurred(&self) -> bool {
        self.exception.is_some()
    }
}

impl JavaMainThread {
    pub fn run(&self) {
        let main = runtime::require_class3(None, self.main_class.as_bytes()).unwrap();
        let main = main.lock().unwrap();
        let main_method = main
            .get_static_method("([Ljava/lang/String;)V", "main")
            .unwrap();

        let mut args = self.args.as_ref().and_then(|args| {
            Some(
                args.iter()
                    .map(|it| {
                        let v = Arc::new(Vec::from(it.as_bytes()));
                        OopDesc::new_str(v)
                    })
                    .collect(),
            )
        });

        let mut jt = Arc::new(JavaThread::new(main_method, args));
        JavaThread::run(jt);
    }
}
