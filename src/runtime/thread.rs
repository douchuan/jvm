use crate::classfile::attr_info::AttrType::Exceptions;
use crate::classfile::{self, signature};
use crate::oop::{self, consts, InstOopDesc, Oop};
use crate::runtime::{self, init_vm, require_class3, FrameRef, JavaCall};
use crate::types::{ClassRef, MethodIdRef};
use crate::util::{self, new_field_id, new_method_id};
use std::borrow::BorrowMut;
use std::sync::{Arc, Mutex};

pub struct JavaThread {
    pub frames: Vec<FrameRef>,
    in_safe_point: bool,

    pub java_thread_obj: Option<Oop>,
    ex: Option<Oop>,
}

pub struct JavaMainThread {
    pub class: String,
    pub args: Option<Vec<String>>,
    dispatch_uncaught_exception_called: bool,
}

impl JavaThread {
    pub fn new() -> Self {
        Self {
            frames: Vec::new(),
            in_safe_point: false,

            java_thread_obj: None,
            ex: None,
        }
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

/*
impl JavaThread {

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
                            "continue: {}:{}:{}, rettype={:?}",
                            String::from_utf8_lossy(cls_name.as_slice()),
                            String::from_utf8_lossy(frame.mir.method.name.as_slice()),
                            String::from_utf8_lossy(frame.mir.method.desc.as_slice()),
                            last_return_type,
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
}
*/

impl JavaMainThread {
    pub fn new(class: String, args: Option<Vec<String>>) -> Self {
        Self {
            class,
            args,
            dispatch_uncaught_exception_called: false,
        }
    }

    pub fn run(&mut self) {
        let mut jt = JavaThread::new();

        info!("init vm start...");
        init_vm::initialize_jvm(&mut jt);
        info!("init vm end");

        let main_class = oop::class::load_and_init(&mut jt, self.class.as_bytes());

        let mir = {
            let cls = main_class.read().unwrap();

            /*
            为了避免"<clinit>"被执行 2 次，这里不允许用路径分隔符

            假如一个自定义类叫做"MyFile", 而且包含"<clinit>"， 即包括如下初始化信息：
                private static File gf = newFile();

            如果文件名包含路径信息：
              xx1: oop::class::load_and_init，加载"test/MyFile"初始化，并调用"<clinit>"
              xx2: 之后vm执行，调用invokestatic，加载"MyFile"初始化，并调用"<clinit>"

            实际，这时两个是同一个类，只允许加载1次
            */
            if self.class.as_bytes() != cls.name.as_slice() {
                panic!("Error: Could not find or load main class {}", self.class);
            }

            let id = util::new_method_id(b"main", b"([Ljava/lang/String;)V");
            cls.get_static_method(id)
        };

        match mir {
            Ok(mir) => {
                let arg = self.build_main_arg(&mut jt);
                let area = runtime::DataArea::new(0, 1);
                {
                    area.borrow_mut().stack.push_ref(arg);
                }
                match JavaCall::new(&mut jt, &area, mir) {
                    Ok(mut jc) => jc.invoke(&mut jt, Some(&area), true),
                    _ => unreachable!(),
                }
            }
            _ => unimplemented!(),
        }

        if jt.ex.is_some() {
            self.uncaught_ex(&mut jt, main_class);
        }
    }
}

impl JavaMainThread {
    fn build_main_arg(&self, jt: &mut JavaThread) -> Oop {
        let args = match &self.args {
            Some(args) => args
                .iter()
                .map(|it| util::oop::new_java_lang_string2(jt, it))
                .collect(),
            None => vec![],
        };

        //build ArrayOopDesc
        let ary_str_class = runtime::require_class3(None, b"[Ljava/lang/String;").unwrap();
        Oop::new_ref_ary2(ary_str_class, args)
    }

    fn uncaught_ex(&mut self, jt: &mut JavaThread, main_cls: ClassRef) {
        if self.dispatch_uncaught_exception_called {
            self.uncaught_ex_internal(jt);
        } else {
            self.dispatch_uncaught_exception_called = true;
            self.call_dispatch_uncaught_exception(jt, main_cls);
        }
    }

    fn call_dispatch_uncaught_exception(&mut self, jt: &mut JavaThread, main_cls: ClassRef) {
        let v = jt.java_thread_obj.clone();
        match v {
            Some(v) => {
                let cls = {
                    let v = util::oop::extract_ref(&v);
                    let v = v.read().unwrap();
                    match &v.v {
                        oop::RefKind::Inst(inst) => inst.class.clone(),
                        _ => unreachable!(),
                    }
                };

                let mir = {
                    let cls = cls.read().unwrap();
                    let id =
                        new_method_id(b"dispatchUncaughtException", b"(Ljava/lang/Throwable;)V");
                    cls.get_this_class_method(id)
                };

                match mir {
                    Ok(mir) => {
                        let ex = { jt.take_ex().unwrap() };
                        let args = vec![v.clone(), ex];
                        let mut jc = JavaCall::new_with_args(jt, mir, args);
                        let area = runtime::DataArea::new(0, 0);
                        jc.invoke(jt, Some(&area), false);
                    }
                    _ => self.uncaught_ex_internal(jt),
                }
            }

            None => self.uncaught_ex_internal(jt),
        }
    }

    fn uncaught_ex_internal(&mut self, jt: &mut JavaThread) {
        let ex = { jt.take_ex().unwrap() };

        let cls = {
            match &ex {
                Oop::Ref(v) => {
                    let v = v.read().unwrap();
                    match &v.v {
                        oop::RefKind::Inst(inst) => inst.class.clone(),
                        _ => unreachable!(),
                    }
                }
                _ => unreachable!(),
            }
        };
        let detail_message = {
            let v = {
                let cls = cls.read().unwrap();
                let id = cls.get_field_id(b"detailMessage", b"Ljava/lang/String;", false);
                cls.get_field_value(&ex, id)
            };

            util::oop::extract_str(&v)
        };
        let name = {
            let cls = cls.read().unwrap();
            cls.name.clone()
        };

        let cls_name = String::from_utf8_lossy(name.as_slice());
        error!("Name={}, detailMessage={}", cls_name, detail_message);
    }
}
