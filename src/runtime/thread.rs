use crate::classfile::{self, signature};
use crate::oop::{self, consts, InstOopDesc, MethodIdRef, OopDesc};
use crate::runtime::{self, Frame, JavaCall, Local, Stack};
use std::borrow::BorrowMut;
use std::sync::{Arc, Mutex};

pub type JavaThreadRef = Arc<JavaThread>;

pub struct JavaThread {
    pub frames: Vec<Frame>,
    in_safe_point: bool,

    java_thread_obj: Option<Arc<OopDesc>>,
    pub exception: Option<Arc<OopDesc>>,
}

pub struct JavaMainThread {
    pub class: String,
    pub args: Option<Vec<String>>,
}

impl JavaThread {
    pub fn new() -> Self {
        Self {
            frames: Vec::new(),
            in_safe_point: false,

            java_thread_obj: None,
            exception: None,
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
        let mir = {
            let class = runtime::require_class3(None, self.class.as_bytes()).unwrap();
            let class = class.lock().unwrap();
            class.get_static_method(b"([Ljava/lang/String;)V", b"main")
        };

        let jtr = Arc::new(JavaThread::new());
        let mut stack = self.build_stack();
        let jc = JavaCall::new(jtr, &mut stack, mir);
        jc.unwrap().invoke(&mut stack);
        info!("stack = {:?}", stack);
    }
}

impl JavaMainThread {
    fn build_stack(&self) -> Stack {
        //args array => Vec<Arc<OopDesc>>
        let args = match &self.args {
            Some(args) => args
                .iter()
                .map(|it| {
                    let v = Arc::new(Vec::from(it.as_bytes()));
                    OopDesc::new_str(v)
                })
                .collect(),
            None => vec![consts::get_null()],
        };

        //build ArrayOopDesc
        let string_class = runtime::require_class3(None, classfile::consts::J_STRING).unwrap();
        let ary = oop::ArrayOopDesc {
            class: string_class,
            elements: args,
        };
        let arg = OopDesc::new_ary(ary);

        //push to stack
        let mut stack = Stack::new(1);
        stack.push_ref(arg);

        stack
    }
}
