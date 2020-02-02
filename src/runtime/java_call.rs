use crate::classfile::consts;
use crate::classfile::signature::{self, MethodSignature, Type as ArgType};
use crate::native;
use crate::oop::{self, ClassRef, MethodIdRef, Oop, OopDesc, OopRef};
use crate::runtime::{self, thread, Frame, JavaThread, Stack};
use std::borrow::BorrowMut;
use std::sync::{Arc, Mutex};

pub struct JavaCall {
    pub mir: MethodIdRef,
    pub args: Vec<OopRef>,
    pub return_type: ArgType,
}

impl JavaCall {
    pub fn new_with_args(jt: &mut JavaThread, mir: MethodIdRef, args: Vec<OopRef>) -> Self {
        let sig = MethodSignature::new(mir.method.desc.as_slice());
        let return_type = sig.retype.clone();
        Self {
            mir,
            args,
            return_type
        }
    }

    pub fn new(jt: &mut JavaThread, stack: &mut Stack, mir: MethodIdRef) -> Result<JavaCall, ()> {
//        let class_name = { mir.method.class.lock().unwrap().name.clone() };

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
            let v = stack.pop_ref();

            //check NPE
            {
                let v = v.lock().unwrap();
                match &v.v {
                    Oop::Null => {
                        jt.throw_ext(consts::J_NPE, false);
                        //todo: caller should call handle_exception
                        return Err(());
                    }
                    _ => (),
                }
            }

            args.insert(0, v);
        }

        //        trace!("class name={}, method name ={} desc={} static={}, native={}",
        //               String::from_utf8_lossy(class_name.as_slice()),
        //               String::from_utf8_lossy(mir.method.name.as_slice()),
        //               String::from_utf8_lossy(mir.method.desc.as_slice()),
        //               mir.method.is_static(),
        //               mir.method.is_native());

        Ok(Self {
            mir,
            args,
            return_type,
        })
    }

    pub fn invoke(&mut self, jt: &mut JavaThread, stack: &mut Stack) {
        if self.mir.method.is_native() {
            self.invoke_native(jt, stack);
        } else {
            self.invoke_java(jt, stack);
        }
    }

    pub fn invoke_java(&mut self, jt: &mut JavaThread, stack: &mut Stack) {
        self.prepare_sync();

        match self.prepare_frame(jt) {
            Ok(frame) => {
                let mut frame = Arc::new(Mutex::new(frame));
                jt.frames.push(frame.clone());

                //exec interp
                {
                    match frame.try_lock() {
                        Ok(mut frame) => {
                            frame.exec_interp(jt);
                            self.set_return(jt, stack, frame.return_v.clone());
                        }
                        _ => unreachable!(),
                    }
                }

                let _frame = jt.frames.pop().unwrap();
            }

            _ => (),
        }

        self.fin_sync();
    }

    pub fn invoke_native(&mut self, jt: &mut JavaThread, stack: &mut Stack) {
        self.prepare_sync();

        let package = {
            let cls = self.mir.method.class.lock().unwrap();
            cls.name.clone()
        };
        let desc = self.mir.method.desc.clone();
        let name = self.mir.method.name.clone();
        let method = native::find_symbol(package.as_slice(), desc.as_slice(), name.as_slice());
        let v = match method {
            Some(method) => {
                let class = self.mir.method.class.clone();
                let env = native::new_jni_env(jt, class);
                method.invoke(env, self.args.clone())
            }
            None => unreachable!(),
        };

        match v {
            Ok(v) => self.set_return(jt, stack, v),
            Err(exception) => unimplemented!()
        }

        self.fin_sync();
    }
}

impl JavaCall {
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

    fn prepare_frame(&mut self, thread: &mut JavaThread) -> Result<Frame, ()> {
        if thread.frames.len() >= runtime::consts::THREAD_MAX_STACK_FRAMES {
            thread.throw_ext(consts::J_SOE, false);
            return Err(());
        }

        let mut frame = Frame::new(self.mir.clone());

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

        return Ok(frame);
    }

    fn set_return(&mut self, thread: &mut JavaThread, stack: &mut Stack, v: Option<OopRef>) {
        if !thread.is_exception_occurred() {
            match self.return_type {
                ArgType::Char | ArgType::Int | ArgType::Boolean => {
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
                ArgType::Object(_) | ArgType::Array(_, _) => {
                    let v = v.unwrap();
                    stack.push_ref(v);
                }
                ArgType::Void => (),
                _ => unreachable!(),
            }
        }
    }
}

fn build_method_args(stack: &mut Stack, sig: MethodSignature) -> Vec<OopRef> {
    //Note: iter args by reverse, because of stack
    sig.args
        .iter()
        .rev()
        .map(|t| {
            match t {
                ArgType::Boolean | ArgType::Int => {
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
                ArgType::Object(_) | ArgType::Array(_, _) => stack.pop_ref(),
                t => unreachable!("t = {:?}", t),
            }
        })
        .collect()
}
