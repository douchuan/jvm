use crate::classfile::{self, signature};
use crate::oop::{self, consts, ClassRef, InstOopDesc, MethodIdRef, OopDesc, OopRef};
use crate::runtime::{self, init_vm, require_class3, FrameRef, JavaCall, Local, Stack};
use std::borrow::BorrowMut;
use std::sync::{Arc, Mutex};

pub struct JavaThread {
    pub frames: Vec<FrameRef>,
    in_safe_point: bool,

    pub java_thread_obj: Option<OopRef>,
    pub exception: Option<OopRef>,
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

    pub fn set_java_thread_obj(&mut self, obj: OopRef) {
        self.java_thread_obj = Some(obj);
    }

    pub fn run(&mut self) {
        //todo: impl
    }

    pub fn throw_ext(&mut self, ext: &[u8], rethrow: bool) {
        //todo: impl
    }

    pub fn throw_ext_with_msg(&mut self, ext: &[u8], rethrow: bool, msg: String) {
        let cls = require_class3(None, ext).unwrap();
        let ctor = {
            let cls = cls.lock().unwrap();
            cls.get_this_class_method(b"(Ljava/lang/String;)V", b"<init>")
                .unwrap()
        };
        let exception = OopDesc::new_inst(cls.clone());
        let msg = Arc::new(Box::new(Vec::from(msg.as_str())));
        let args = vec![exception.clone(), OopDesc::new_str(msg)];
        let mut jc = JavaCall::new_with_args(self, ctor, args);
        let mut stack = Stack::new(0);
        jc.invoke(self, &mut stack);
        self.exception = Some(exception);
    }

    pub fn throw_ext_with_msg2(&mut self, ext: &[u8], rethrow: bool, msg: &[u8]) {
        //todo: impl
    }

    pub fn try_handle_exception(&mut self, ex: OopRef) -> i32 {
        //todo: impl
        unimplemented!()
    }

    pub fn clear_ext(&mut self) {
        self.exception = None;
    }

    pub fn is_exception_occurred(&self) -> bool {
        self.exception.is_some()
    }
}

impl JavaMainThread {
    pub fn run(&self) {
        let mut jt = JavaThread::new();

        init_vm::initialize_jvm(&mut jt);

        let mir = {
            let class = runtime::require_class3(None, self.class.as_bytes()).unwrap();
            let class = class.lock().unwrap();
            class.get_static_method(b"([Ljava/lang/String;)V", b"main")
        };

        match mir {
            Ok(mir) => {
                let mut stack = self.build_stack();
                let jc = JavaCall::new(&mut jt, &mut stack, mir);
                jc.unwrap().invoke(&mut jt, &mut stack);
                info!("stack = {:?}", stack);
            }
            _ => unimplemented!(),
        }
    }
}

impl JavaMainThread {
    fn build_stack(&self) -> Stack {
        let args = match &self.args {
            Some(args) => args
                .iter()
                .map(|it| {
                    let v = Arc::new(Box::new(Vec::from(it.as_bytes())));
                    OopDesc::new_str(v)
                })
                .collect(),
            None => vec![consts::get_null()],
        };

        //build ArrayOopDesc
        let string_class = runtime::require_class3(None, b"[Ljava/lang/String;").unwrap();
        let arg = OopDesc::new_ary2(string_class, args);

        //push to stack
        let mut stack = Stack::new(1);
        stack.push_ref(arg);

        stack
    }
}
