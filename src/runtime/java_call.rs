use crate::classfile::consts;
use crate::classfile::signature::{self, MethodSignature, Type as ArgType};
use crate::oop::{ClassRef, MethodIdRef, Oop, OopDesc};
use crate::runtime::{thread, JavaThreadRef, Stack};
use std::sync::Arc;

struct JavaCall {
    jtr: JavaThreadRef,
    mir: MethodIdRef,
    args: Vec<Arc<OopDesc>>,
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

    pub fn invoke_java(&mut self) {
        self.prepare_sync();

        //todo: prepare frame

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
            ArgType::Object(_) | ArgType::Array(_, _) => stack.pop_ref(),
            _ => unreachable!(),
        })
        .collect()
}
