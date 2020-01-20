use crate::classfile::consts;
use crate::classfile::signature::{self, MethodSignature, Type as ArgType};
use crate::oop::{ClassRef, MethodIdRef, Oop, OopDesc};
use crate::runtime::{self, thread, Frame, JavaThreadRef, Stack};
use std::borrow::BorrowMut;
use std::sync::Arc;

pub struct JavaCall {
    pub jtr: JavaThreadRef,
    pub mir: MethodIdRef,
    pub args: Vec<Arc<OopDesc>>,
    pub return_type: ArgType
}

impl JavaCall {
    pub fn new(jtr: JavaThreadRef, stack: &mut Stack, mir: MethodIdRef) -> Result<JavaCall, ()> {
        let sig = MethodSignature::new(mir.method.desc.as_slice());
        let return_type = sig.retype.clone();
        let mut args = build_method_args(stack, sig);

        //insert 'this' value
        let has_this = !mir.method.is_static();
        if has_this {
            let v = stack.pop_ref();

            //check NPE
            match &v.v {
                Oop::Null => {
                    thread::JavaThread::throw_ext(jtr, consts::J_NPE, false);
                    //todo: caller should call handle_exception
                    return Err(());
                }
                _ => (),
            }

            args.insert(0, v);
        }

        Ok(Self { jtr, mir, args, return_type })
    }

    pub fn invoke(&mut self, stack: &mut Stack) {
        if self.mir.method.is_native() {
            unimplemented!()
        } else {
            self.invoke_java(stack);
        }
    }

    pub fn invoke_java(&mut self, stack: &mut Stack) {
        trace!("invoke_java 1");
        self.prepare_sync();

        trace!("invoke_java 2");
        if self.prepare_frame().is_ok() {
            trace!("invoke_java 3");
            //exec interp
            let frame = {
                let jt = Arc::get_mut(&mut self.jtr).unwrap();
                jt.frames.last_mut().unwrap()
            };

            trace!("invoke_java 4");
            frame.exec_interp();

            let frame = {
                let jt = Arc::get_mut(&mut self.jtr).unwrap();
                jt.frames.pop().unwrap()
            };

            self.set_return(stack, frame.return_v);
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
                let v = Arc::get_mut(&mut v).unwrap();
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
                let v = Arc::get_mut(&mut v).unwrap();
                v.monitor_exit();
            }
        }
    }

    fn prepare_frame(&mut self) -> Result<(), ()> {
        if self.jtr.frames.len() >= runtime::consts::THREAD_MAX_STACK_FRAMES {
            thread::JavaThread::throw_ext(self.jtr.clone(), consts::J_SOE, false);
            return Err(());
        }

        let mut frame = Frame::new(self.jtr.clone(), self.mir.clone());

        //JVM spec, 2.6.1
        let locals = &mut frame.local;
        let mut slot_pos: usize = 0;
        self.args.iter().for_each(|v| {
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
                    locals.set_ref(slot_pos, v.clone());
                    1
                }
            };

            slot_pos += step;
        });

        let jt = Arc::get_mut(&mut self.jtr).unwrap();
        jt.frames.push(frame);

        return Ok(());
    }

    fn set_return(&mut self, stack: &mut Stack, v: Option<Arc<OopDesc>>) {
        if !self.jtr.is_exception_occurred() {
            match self.return_type {
                ArgType::Int => {
                    match v {
                        Some(v) => {
                            match v.v {
                                Oop::Int(v) => stack.push_int(v),
                                _ => unreachable!(),
                            }
                        },
                        None => unreachable!(),
                    }
                }
                ArgType::Long => {
                    match v {
                        Some(v) => {
                            match v.v {
                                Oop::Long(v) => stack.push_long(v),
                                _ => unreachable!()
                            }
                        },
                        None => unreachable!(),
                    }
                }
                ArgType::Float => {
                    match v {
                        Some(v) => {
                            match v.v {
                                Oop::Float(v) => stack.push_float(v),
                                _ => unreachable!()
                            }
                        },
                        None => unreachable!()
                    }
                }
                ArgType::Double => {
                    match v {
                        Some(v) => {
                            match v.v {
                                Oop::Double(v) => stack.push_double(v),
                                _ => unreachable!()
                            }
                        },
                        None => unreachable!()
                    }
                }
                ArgType::Object(_) | ArgType::Array(_, _) => {
                    match v {
                        Some(v) => stack.push_ref(v),
                        None => unreachable!()
                    }
                }
                ArgType::Void => (),
                _ => unreachable!()
            }
        }
    }
}

fn build_method_args(stack: &mut Stack, sig: MethodSignature) -> Vec<Arc<OopDesc>> {
    sig.args
        .iter()
        .map(|t| match t {
            ArgType::Int => {
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
            _ => stack.pop_ref(),
        })
        .collect()
}
