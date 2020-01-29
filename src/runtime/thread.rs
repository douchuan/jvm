use crate::classfile::{self, signature};
use crate::oop::{self, consts, InstOopDesc, MethodIdRef, OopDesc, OopRef};
use crate::runtime::{self, Frame, JavaCall, Local, Stack};
use std::borrow::BorrowMut;
use std::sync::{Arc, Mutex};

pub struct JavaThread {
    pub frames: Vec<Arc<Mutex<Frame>>>,
    in_safe_point: bool,

    java_thread_obj: Option<OopRef>,
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

    pub fn run(&mut self) {
        //todo: impl
    }

    pub fn throw_ext(&mut self, ext: &[u8], rethrow: bool) {
        //todo: impl
    }

    pub fn throw_ext_with_msg(&mut self, ext: &[u8], rethrow: bool, msg: String) {
        //todo: impl
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

        let classes = vec![
            classfile::consts::J_CLASS,
            classfile::consts::J_OBJECT,
            classfile::consts::J_STRING,
            classfile::consts::J_CLONEABLE,
            classfile::consts::J_SERIALIZABLE,
            classfile::consts::J_NPE,
            classfile::consts::J_ARRAY_INDEX_OUT_OF_BOUNDS,
            classfile::consts::J_CLASS_NOT_FOUND,
            classfile::consts::J_INTERNAL_ERROR,
            classfile::consts::J_IOEXCEPTION,
            classfile::consts::J_SYSTEM,
        ];
        classes.iter().for_each(|c| {
            let class = runtime::require_class3(None, *c);
            let class = class.unwrap();
            {
                let mut class = class.lock().unwrap();
                class.init_class(&mut jt);
                trace!("finish init_class: {}", String::from_utf8_lossy(*c));
            }
            oop::class::init_class_fully(&mut jt, class);
            trace!("finish init_class_fully: {}", String::from_utf8_lossy(*c));
        });

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
            _ => unimplemented!()
        }


    }
}

impl JavaMainThread {
    fn build_stack(&self) -> Stack {
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
        let string_class = runtime::require_class3(None, b"[Ljava/lang/String;").unwrap();
        let arg = OopDesc::new_ary2(string_class, args);

        //push to stack
        let mut stack = Stack::new(1);
        stack.push_ref(arg);

        stack
    }
}
