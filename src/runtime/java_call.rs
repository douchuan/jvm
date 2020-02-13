use crate::classfile::consts;
use crate::classfile::signature::{self, MethodSignature, Type as ArgType};
use crate::native;
use crate::oop::{self, ClassRef, MethodIdRef, Oop, OopDesc, OopRef, ValueType};
use crate::runtime::{self, frame::Frame, thread, FrameRef, JavaThread, Stack};
use crate::util;
use std::borrow::BorrowMut;
use std::sync::{Arc, Mutex};

pub struct JavaCall {
    pub mir: MethodIdRef,
    pub args: Vec<OopRef>,
    pub return_type: ArgType,
}

pub fn invoke_ctor(jt: &mut JavaThread, cls: ClassRef, desc: &[u8], args: Vec<OopRef>) {
    let ctor = {
        let cls = cls.lock().unwrap();
        let id = util::new_method_id(b"<init>", desc);
        cls.get_this_class_method(id).unwrap()
    };

    let mut jc = JavaCall::new_with_args(jt, ctor, args);
    let mut stack = Stack::new(0);
    jc.invoke(jt, &mut stack, false);
}

impl JavaCall {
    pub fn new_with_args(jt: &mut JavaThread, mir: MethodIdRef, args: Vec<OopRef>) -> Self {
        let sig = MethodSignature::new(mir.method.desc.as_slice());
        let return_type = sig.retype.clone();
        Self {
            mir,
            args,
            return_type,
        }
    }

    pub fn new(jt: &mut JavaThread, stack: &mut Stack, mir: MethodIdRef) -> Result<JavaCall, ()> {
        let sig = MethodSignature::new(mir.method.desc.as_slice());
        let return_type = sig.retype.clone();

        let mut args = build_method_args(stack, sig);
        args.reverse();

        /*
        let mut xx = Vec::with_capacity(args.len());
        args.iter().for_each(|it| {
            let it = it.lock().unwrap();
            match it.v {
                Oop::Int(_) => xx.push("int"),
                Oop::Str(_) => xx.push("str"),
                Oop::Array(_) => xx.push("ary"),
                Oop::Inst(_) => xx.push("obj"),
                Oop::Null => xx.push("null"),
                Oop::Double(_) => xx.push("double"),
                Oop::Long(_) => xx.push("long"),
                Oop::Float(_) => xx.push("float"),
                Oop::Mirror(_) => xx.push("mirror")
            }
        });
        trace!("xx = {}", xx.join(":"));
        */

        //insert 'this' value
        let has_this = !mir.method.is_static();
        if has_this {
            let this = stack.pop_ref();
            let this_back = this.clone();

            //check NPE
            {
                let v = this.lock().unwrap();
                match &v.v {
                    Oop::Null => {
                        let cls_name = {
                            let cls = mir.method.class.lock().unwrap();
                            cls.name.clone()
                        };

                        error!("Java new failed, null this: {}:{}, this={:?}",
                            String::from_utf8_lossy(cls_name.as_slice()),
                            String::from_utf8_lossy(mir.method.get_id().as_slice()),
                            v);
                        jt.throw_ex(consts::J_NPE);
                        return Err(());
                    }
                    _ => (),
                }
            }

            args.insert(0, this);
        }

        Ok(Self {
            mir,
            args,
            return_type,
        })
    }
}

impl JavaCall {
    pub fn invoke(&mut self, jt: &mut JavaThread, stack: &mut Stack, force_no_resolve: bool) {
        self.debug();

        if self.mir.method.is_native() {
            jt.callers.push(self.mir.clone());
            self.invoke_native(jt, stack);
        } else {
            self.resolve_virtual_method(force_no_resolve);
            jt.callers.push(self.mir.clone());
            self.invoke_java(jt, stack);
        }

        jt.callers.pop();

        jt.handle_ex();
    }
}

impl JavaCall {
    fn invoke_java(&mut self, jt: &mut JavaThread, stack: &mut Stack) {
        self.prepare_sync();

        match self.prepare_frame(jt) {
            Ok(frame) => {
                jt.frames.push(frame.clone());

                match frame.try_lock() {
                    Ok(mut frame) => {
                        frame.interp(jt);

                        if !jt.is_meet_ex() {
                            set_return(stack, self.return_type.clone(), frame.return_v.clone());
                            let _ = jt.frames.pop();
                        } else {
                            info!("ignored pop frame_id = {}", frame.frame_id);
                        }
                    }
                    _ => unreachable!(),
                }
            }

            _ => (),
        }

        self.fin_sync();
    }

    fn invoke_native(&mut self, jt: &mut JavaThread, stack: &mut Stack) {
        self.prepare_sync();

        let package = {
            let cls = self.mir.method.class.lock().unwrap();
            cls.name.clone()
        };
        let desc = self.mir.method.desc.clone();
        let name = self.mir.method.name.clone();
        let method = native::find_symbol(package.as_slice(), name.as_slice(), desc.as_slice());
        let v = match method {
            Some(method) => {
                let class = self.mir.method.class.clone();
                let env = native::new_jni_env(jt, class);
                method.invoke(jt, env, self.args.clone())
            }
            None => unreachable!("NotFound native method"),
        };

        match v {
            Ok(v) => {
                if !jt.is_meet_ex() {
                    set_return(stack, self.return_type.clone(), v)
                }
            }
            Err(ex) => {
                //fixme:
                //把charsets.jar去掉，会让代码走到这里
                //ex is putted in jt.ex
                jt.set_ex(Some(ex));
            }
        }

        self.fin_sync();
    }

    fn prepare_sync(&mut self) {
        if self.mir.method.is_synchronized() {
            if self.mir.method.is_static() {
                let mut class = self.mir.method.class.lock().unwrap();
                class.monitor_enter();
            } else {
                let mut v = self.args.first_mut().unwrap();
                let mut v = v.lock().unwrap();
                v.monitor_enter();
            }
        }
    }

    fn fin_sync(&mut self) {
        if self.mir.method.is_synchronized() {
            if self.mir.method.is_static() {
                let mut class = self.mir.method.class.lock().unwrap();
                class.monitor_exit();
            } else {
                let mut v = self.args.first_mut().unwrap();
                let mut v = v.lock().unwrap();
                v.monitor_exit();
            }
        }
    }

    fn prepare_frame(&mut self, thread: &mut JavaThread) -> Result<FrameRef, ()> {
        if thread.frames.len() >= runtime::consts::THREAD_MAX_STACK_FRAMES {
            //todo: test me
            thread.throw_ex(consts::J_SOE);
            return Err(());
        }

        let frame_id = thread.frames.len() + 1;
        let mut frame = Frame::new(self.mir.clone(), frame_id);

        //JVM spec, 2.6.1
        let locals = &mut frame.local;
        let mut slot_pos: usize = 0;
        self.args.iter().for_each(|v| {
            let v_ref = v.clone();
            let v = v.lock().unwrap();
            let step = match &v.v {
                Oop::Int(v) => {
                    locals.set_int(slot_pos, *v);
                    1
                }
                Oop::Float(v) => {
                    locals.set_float(slot_pos, *v);
                    1
                }
                Oop::Double(v) => {
                    locals.set_double(slot_pos, *v);
                    2
                }
                Oop::Long((v)) => {
                    locals.set_long(slot_pos, *v);
                    2
                }
                _ => {
                    locals.set_ref(slot_pos, v_ref);
                    1
                }
            };

            slot_pos += step;
        });

        let frame_ref = new_sync_ref!(frame);
        return Ok(frame_ref);
    }

    fn resolve_virtual_method(&mut self, force_no_resolve: bool) {
        let resolve_twice = if force_no_resolve {
            false
        } else {
            self.mir.method.is_abstract()
                || (self.mir.method.is_public() && !self.mir.method.is_final())
        };
        if resolve_twice {
            let this = self.args[0].clone();
            let this = this.lock().unwrap();
            match &this.v {
                Oop::Inst(inst) => {
                    let cls = inst.class.clone();
                    let cls = cls.lock().unwrap();
                    let id = self.mir.method.get_id();
                    self.mir = cls.get_virtual_method(id).unwrap();
                }
                _ => (),
            };
        }
    }

    fn debug(&self) {
        let cls_name = { self.mir.method.class.lock().unwrap().name.clone() };
        let name = self.mir.method.name.clone();
        let desc = self.mir.method.desc.clone();
        let id =
            vec![cls_name.as_slice(), name.as_slice(), desc.as_slice()].join(util::PATH_DELIMITER);
        info!(
            "invoke method = {} static={} native={}",
            String::from_utf8_lossy(&id),
            self.mir.method.is_static(),
            self.mir.method.is_native()
        );
    }
}

fn build_method_args(stack: &mut Stack, sig: MethodSignature) -> Vec<OopRef> {
    //Note: iter args by reverse, because of stack
    sig.args
        .iter()
        .rev()
        .map(|t| match t {
            ArgType::Boolean | ArgType::Int | ArgType::Char => {
                let v = stack.pop_int();
                OopDesc::new_int(v)
            }
            ArgType::Long => {
                let v = stack.pop_long();
                OopDesc::new_long(v)
            }
            ArgType::Float => {
                let v = stack.pop_float();
                OopDesc::new_float(v)
            }
            ArgType::Double => {
                let v = stack.pop_double();
                OopDesc::new_double(v)
            }
            ArgType::Object(_) | ArgType::Array(_) => stack.pop_ref(),
            t => unreachable!("t = {:?}", t),
        })
        .collect()
}

pub fn set_return(stack: &mut Stack, return_type: ArgType, v: Option<OopRef>) {
    match return_type {
        ArgType::Byte | ArgType::Char | ArgType::Int | ArgType::Boolean => {
            let v = v.unwrap();
            let v = v.lock().unwrap();
            match v.v {
                Oop::Int(v) => stack.push_int(v),
                _ => unreachable!(),
            }
        }
        ArgType::Long => {
            let v = v.unwrap();
            let v = v.lock().unwrap();
            match v.v {
                Oop::Long(v) => stack.push_long(v),
                _ => unreachable!(),
            }
        }
        ArgType::Float => {
            let v = v.unwrap();
            let v = v.lock().unwrap();
            match v.v {
                Oop::Float(v) => stack.push_float(v),
                _ => unreachable!(),
            }
        }
        ArgType::Double => {
            let v = v.unwrap();
            let v = v.lock().unwrap();
            match v.v {
                Oop::Double(v) => stack.push_double(v),
                _ => unreachable!(),
            }
        }
        ArgType::Object(_) | ArgType::Array(_) => {
            let v = v.unwrap();
            stack.push_ref(v);
        }
        ArgType::Void => (),
        _ => unreachable!(),
    }
}
