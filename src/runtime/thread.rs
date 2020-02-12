use crate::classfile::attr_info::AttrType::Exceptions;
use crate::classfile::{self, signature};
use crate::oop::{self, consts, ClassRef, InstOopDesc, MethodIdRef, OopDesc, OopRef};
use crate::runtime::{self, init_vm, require_class3, Exception, FrameRef, JavaCall, Local, Stack};
use crate::util;
use crate::util::new_field_id;
use std::borrow::BorrowMut;
use std::sync::{Arc, Mutex};

pub struct JavaThread {
    pub frames: Vec<FrameRef>,
    in_safe_point: bool,

    pub java_thread_obj: Option<OopRef>,
    ex: Option<Exception>,

    pub callers: Vec<MethodIdRef>,
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
            ex: None,

            callers: vec![],
        }
    }

    pub fn set_java_thread_obj(&mut self, obj: OopRef) {
        self.java_thread_obj = Some(obj);
    }
}

//exception
impl JavaThread {
    pub fn throw_ex(&mut self, ex: &'static [u8]) {
        let ex = Vec::from(ex);
        let ex = Exception {
            cls_name: new_ref!(ex),
            msg: None,
            ex_oop: None,
        };
        self.ex = Some(ex);
    }

    pub fn set_ex(&mut self, ex: Option<Exception>) {
        self.ex = ex;
    }

    pub fn is_meet_ex(&self) -> bool {
        self.ex.is_some()
    }

    pub fn handle_ex(&mut self) {
        if self.is_meet_ex() && self.is_invoke_ended() {
            //consume the ex
            let ex = self.ex.take().unwrap();
            info!(
                "handle exception = {}, msg = {:?}",
                String::from_utf8_lossy(ex.cls_name.as_slice()),
                ex.msg
            );

            let ex_oop = {
                if ex.ex_oop.is_none() {
                    self.build_ex_oop(ex)
                } else {
                    ex.ex_oop.clone().unwrap()
                }
            };

            Self::debug_ex(ex_oop.clone());

            self.do_handle_ex(ex_oop);
        }
    }
}

impl JavaThread {
    //all frames lock released
    fn is_invoke_ended(&self) -> bool {
        self.frames.iter().all(|f| f.try_lock().is_ok())
    }

    fn build_ex_oop(&mut self, mut ex: Exception) -> OopRef {
        let cls = require_class3(None, ex.cls_name.as_slice()).unwrap();
        let ex_obj = OopDesc::new_inst(cls.clone());

        //invoke ctor
        match &ex.msg {
            Some(msg) => {
                //with 'String' arg ctor
                let msg = Vec::from(msg.as_str());
                let msg = new_ref!(msg);
                let args = vec![ex_obj.clone(), OopDesc::new_str(msg)];
                runtime::java_call::invoke_ctor(self, cls.clone(), b"(Ljava/lang/String;)V", args);
            }
            None => {
                //No arg ctor
                let args = vec![ex_obj.clone()];
                runtime::java_call::invoke_ctor(self, cls.clone(), b"()V", args);
            }
        }

        ex_obj
    }

    fn do_handle_ex(&mut self, ex: OopRef) {
        self.debug_frames();

        //guard last frame meet exception
        let last_frame = self.frames.last().unwrap();
        {
            match last_frame.try_lock() {
                Ok(mut frame) => {
                    assert!(frame.meet_ex_here);
                }
                _ => unreachable!(),
            }
        }

        let mut re_throw_ex = Some(ex);
        let mut last_return_value = None;
        let mut last_return_type = None;
        loop {
            let frame = self.frames.pop();
            if frame.is_none() {
                break;
            }
            let frame = frame.unwrap();

            self.frames.push(frame.clone());

            match frame.try_lock() {
                Ok(mut frame) => match re_throw_ex.clone() {
                    Some(ex) => {
                        frame.handle_exception(self, ex);

                        if self.is_meet_ex() {
                            unimplemented!("meet ex again")
                        } else {
                            re_throw_ex = frame.re_throw_ex.take();

                            if re_throw_ex.is_none() {
                                frame.interp(self);
                                let sig = signature::MethodSignature::new(
                                    frame.mir.method.desc.as_slice(),
                                );
                                last_return_type = Some(sig.retype.clone());
                                last_return_value = frame.return_v.clone();
                                re_throw_ex = None;
                            }
                        }
                    }
                    None => {
                        let cls_name = {
                            let cls = frame.mir.method.class.lock().unwrap();
                            cls.name.clone()
                        };
                        info!(
                            "continue: {}:{}:{}, rettype={:?}, last_return_value={:?}",
                            String::from_utf8_lossy(cls_name.as_slice()),
                            String::from_utf8_lossy(frame.mir.method.name.as_slice()),
                            String::from_utf8_lossy(frame.mir.method.desc.as_slice()),
                            last_return_type,
                            last_return_value
                        );
                        runtime::java_call::set_return(
                            &mut frame.stack,
                            last_return_type.clone().unwrap(),
                            last_return_value.clone(),
                        );
                        frame.interp(self);

                        if self.is_meet_ex() {
                            unimplemented!("meet ex again")
                        } else {
                            re_throw_ex = frame.re_throw_ex.take();

                            if re_throw_ex.is_none() {
                                let sig = signature::MethodSignature::new(
                                    frame.mir.method.desc.as_slice(),
                                );
                                last_return_type = Some(sig.retype.clone());
                                last_return_value = frame.return_v.clone();
                            }
                        }
                    }
                },
                _ => unreachable!(),
            }

            let _ = self.frames.pop();
        }
    }

    fn debug_frames(&self) {
        trace!("debug frame: count = {}", self.frames.len());
        let mut count = 0;
        for it in self.frames.iter() {
            match it.try_lock() {
                Ok(frame) => {
                    let cls_name = {
                        let cls = frame.mir.method.class.lock().unwrap();
                        cls.name.clone()
                    };
                    let (desc, name) =
                        { (frame.mir.method.desc.clone(), frame.mir.method.name.clone()) };
                    let id = vec![cls_name.as_slice(), desc.as_slice(), name.as_slice()]
                        .join(util::PATH_DELIMITER);
                    trace!(
                        "{} ({}){} ex={}",
                        " ".repeat(count),
                        frame.frame_id,
                        String::from_utf8_lossy(&id),
                        frame.meet_ex_here
                    );
                }
                _ => warn!("locked frame"),
            }

            count += 1;
        }
    }

    fn debug_ex(ex: OopRef) {
        let cls = {
            let v = ex.lock().unwrap();
            match &v.v {
                oop::Oop::Inst(inst) => inst.class.clone(),
                _ => unreachable!(),
            }
        };
        let detail_msg = {
            let detail_message_oop = {
                let cls = cls.lock().unwrap();
                let id = cls.get_field_id(b"detailMessage", b"Ljava/lang/String;", false);
                cls.get_field_value(ex.clone(), id)
            };

            let cls = {
                let v = detail_message_oop.lock().unwrap();
                match &v.v {
                    oop::Oop::Inst(inst) => Some(inst.class.clone()),
                    oop::Oop::Null => None,
                    t => unreachable!("t = {:?}", t),
                }
            };

            if cls.is_none() {
                let v = vec![];
                new_ref!(v)
            } else {
                util::oop::extract_str(detail_message_oop)
            }
        };

        info!("detail={}", String::from_utf8_lossy(detail_msg.as_slice()));
    }
}

impl JavaMainThread {
    pub fn run(&self) {
        let mut jt = JavaThread::new();

        init_vm::initialize_jvm(&mut jt);

        let mir = {
            let class = runtime::require_class3(None, self.class.as_bytes()).unwrap();
            let class = class.lock().unwrap();
            let id = util::new_method_id(b"main", b"([Ljava/lang/String;)V");
            class.get_static_method(id)
        };

        match mir {
            Ok(mir) => {
                let mut stack = self.build_stack();
                match JavaCall::new(&mut jt, &mut stack, mir) {
                    Ok(mut jc) => jc.invoke(&mut jt, &mut stack, true),
                    _ => unreachable!(),
                }
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
