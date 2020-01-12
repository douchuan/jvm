use crate::classfile::consts;
use crate::classfile::signature::{self, MethodSignature, Type as ArgType};
use crate::oop::{ClassRef, MethodIdRef, Oop, OopDesc};
use crate::runtime::{self, thread, JavaThreadRef, Stack, Frame};
use std::sync::Arc;
use std::borrow::BorrowMut;

pub struct JavaCall {
    pub jtr: JavaThreadRef,
    pub mir: MethodIdRef,
    pub args: Vec<Arc<OopDesc>>,
}

impl JavaCall {
    pub fn new(
        jtr: JavaThreadRef,
        stack: &mut Stack,
        mir: MethodIdRef,
    ) -> Result<JavaCall, ()> {
        let sig = MethodSignature::new(mir.method.desc.as_slice());
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

        Ok(Self {
            jtr,
            mir,
            args,
        })
    }

    pub fn invoke_java(&mut self) -> Option<Arc<OopDesc>> {
        let mut r = None;

        self.prepare_sync();

        if self.prepare_frame().is_ok() {

            //exec interp
            let frame= {
                let jt = Arc::get_mut(&mut self.jtr).unwrap();
                jt.frames.last_mut().unwrap()
            };

            frame.exec_interp();

            let frame= {
                let jt = Arc::get_mut(&mut self.jtr).unwrap();
                jt.frames.pop().unwrap()
            };
            r = frame.return_v;
        }

        self.fin_sync();

        r
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
                },
                Oop::Float(v) => {
                    locals.set_float(slot_pos, *v);
                    1
                },
                Oop::Double(v) => {
                    locals.set_double(slot_pos, *v);
                    2
                },
                Oop::Long((v)) => {
                    locals.set_long(slot_pos, *v);
                    2
                },
                _ => {
                    locals.set_ref(slot_pos, v.clone());
                    1
                },
            };

            slot_pos += step;
        });

        let jt = Arc::get_mut(&mut self.jtr).unwrap();
        jt.frames.push(frame);

        return Ok(());
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
