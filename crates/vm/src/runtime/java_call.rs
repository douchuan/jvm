use crate::native;
use crate::oop::{self, Oop, ValueType};
use crate::runtime::{self, exception, frame::Frame, thread, FrameRef, Interp, JavaThread};
use crate::types::{ClassRef, DataAreaRef, MethodIdRef};
use crate::util;
use classfile::{consts as cls_const, SignatureType};
use class_parser::MethodSignature;
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

pub struct JavaCall {
    pub mir: MethodIdRef,
    pub args: Vec<Oop>,
    pub return_type: SignatureType,
}

pub fn invoke_ctor(jt: &mut JavaThread, cls: ClassRef, desc: &[u8], args: Vec<Oop>) {
    let ctor = {
        let cls = cls.read().unwrap();
        let id = util::new_method_id(b"<init>", desc);
        cls.get_this_class_method(id).unwrap()
    };

    let mut jc = JavaCall::new_with_args(jt, ctor, args);
    jc.invoke(jt, None, false);
}

impl JavaCall {
    pub fn new_with_args(jt: &mut JavaThread, mir: MethodIdRef, args: Vec<Oop>) -> Self {
        let sig = MethodSignature::new(mir.method.desc.as_slice());
        let return_type = sig.retype.clone();
        Self {
            mir,
            args,
            return_type,
        }
    }

    pub fn new(
        jt: &mut JavaThread,
        caller: &DataAreaRef,
        mir: MethodIdRef,
    ) -> Result<JavaCall, ()> {
        let sig = MethodSignature::new(mir.method.desc.as_slice());
        let return_type = sig.retype.clone();

        let mut args = build_method_args(caller, sig);
        args.reverse();

        //insert 'this' value
        let has_this = !mir.method.is_static();
        if has_this {
            let this = {
                let mut area = caller.borrow_mut();
                area.stack.pop_ref()
            };

            //check NPE
            match this {
                Oop::Null => {
                    let cls_name = {
                        let cls = mir.method.class.read().unwrap();
                        cls.name.clone()
                    };

                    error!(
                        "Java new failed, null this: {}:{}",
                        String::from_utf8_lossy(cls_name.as_slice()),
                        String::from_utf8_lossy(mir.method.get_id().as_slice())
                    );

                    //Fail fast, avoid a lot of logs, and it is not easy to locate the problem
                    //                        panic!();

                    let ex = exception::new(jt, cls_const::J_NPE, None);
                    jt.set_ex(ex);
                    return Err(());
                }
                _ => (),
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
    pub fn invoke(
        &mut self,
        jt: &mut JavaThread,
        caller: Option<&DataAreaRef>,
        force_no_resolve: bool,
    ) {
        /*
        Do resolve again first, because you can override in a native way such as:
        UnixFileSystem override FileSystem
            public abstract boolean checkAccess(File f, int access);

            public native boolean checkAccess(File f, int access);
        */
        self.resolve_virtual_method(force_no_resolve);
        self.debug();

        if self.mir.method.is_native() {
            self.invoke_native(jt, caller);
        } else {
            self.invoke_java(jt, caller);
        }

        let _ = jt.frames.pop();
    }
}

impl JavaCall {
    fn invoke_java(&mut self, jt: &mut JavaThread, caller: Option<&DataAreaRef>) {
        self.prepare_sync();

        match self.prepare_frame(jt, false) {
            Ok(frame) => {
                jt.frames.push(frame.clone());

                let frame_h = frame.try_read().unwrap();
                let interp = Interp::new(frame_h);
                interp.run(jt);

                if !jt.is_meet_ex() {
                    let return_value = {
                        let frame = frame.try_read().unwrap();
                        let area = frame.area.borrow();
                        area.return_v.clone()
                    };
                    set_return(caller, self.return_type.clone(), return_value);
                }
            }

            Err(ex) => jt.set_ex(ex),
        }

        self.fin_sync();
    }

    fn invoke_native(&mut self, jt: &mut JavaThread, caller: Option<&DataAreaRef>) {
        self.prepare_sync();

        let package = {
            let cls = self.mir.method.class.read().unwrap();
            cls.name.clone()
        };
        let desc = self.mir.method.desc.clone();
        let name = self.mir.method.name.clone();
        let method = native::find_symbol(package.as_slice(), name.as_slice(), desc.as_slice());
        let v = match method {
            Some(method) => {
                let class = self.mir.method.class.clone();
                let env = native::new_jni_env(jt, class);

                match self.prepare_frame(jt, true) {
                    Ok(frame) => {
                        jt.frames.push(frame.clone());
                        method.invoke(jt, env, self.args.clone())
                    }
                    Err(ex) => Err(ex),
                }
            }
            None => panic!(
                "Native method not found: {}:{}:{}",
                unsafe { std::str::from_utf8_unchecked(&package) },
                unsafe { std::str::from_utf8_unchecked(&name) },
                unsafe { std::str::from_utf8_unchecked(&desc) },
            ),
        };

        match v {
            Ok(v) => {
                if !jt.is_meet_ex() {
                    set_return(caller, self.return_type.clone(), v)
                }
            }
            Err(ex) => jt.set_ex(ex),
        }

        self.fin_sync();
    }

    fn prepare_sync(&mut self) {
        if self.mir.method.is_synchronized() {
            if self.mir.method.is_static() {
                let mut class = self.mir.method.class.write().unwrap();
                class.monitor_enter();
            } else {
                let mut v = self.args.first().unwrap();
                let v = util::oop::extract_ref(v);
                let mut v = v.write().unwrap();
                v.monitor_enter();
            }
        }
    }

    fn fin_sync(&mut self) {
        if self.mir.method.is_synchronized() {
            if self.mir.method.is_static() {
                let mut class = self.mir.method.class.write().unwrap();
                class.monitor_exit();
            } else {
                let mut v = self.args.first().unwrap();
                let v = util::oop::extract_ref(v);
                let mut v = v.write().unwrap();
                v.monitor_exit();
            }
        }
    }

    fn prepare_frame(&mut self, thread: &mut JavaThread, is_native: bool) -> Result<FrameRef, Oop> {
        if thread.frames.len() >= runtime::consts::THREAD_MAX_STACK_FRAMES {
            let ex = exception::new(thread, cls_const::J_SOE, None);
            return Err(ex);
        }

        let frame_id = thread.frames.len() + 1;
        let mut frame = Frame::new(self.mir.clone(), frame_id);

        if !is_native {
            //JVM spec, 2.6.1
            let mut area = frame.area.borrow_mut();
            let mut slot_pos: usize = 0;
            self.args.iter().for_each(|v| {
                let step = match v {
                    Oop::Int(v) => {
                        area.local.set_int(slot_pos, *v);
                        1
                    }
                    Oop::Float(v) => {
                        area.local.set_float(slot_pos, *v);
                        1
                    }
                    Oop::Double(v) => {
                        area.local.set_double(slot_pos, *v);
                        2
                    }
                    Oop::Long((v)) => {
                        area.local.set_long(slot_pos, *v);
                        2
                    }
                    _ => {
                        area.local.set_ref(slot_pos, v.clone());
                        1
                    }
                };

                slot_pos += step;
            });
        }

        let frame_ref = new_sync_ref!(frame);
        return Ok(frame_ref);
    }

    fn resolve_virtual_method(&mut self, force_no_resolve: bool) {
        let resolve_again = if force_no_resolve {
            false
        } else {
            //todo: why is the value of 0 possible in acc_flags?
            /*
            This situation occurs when:
            java/util/regex/Matcher.java
            bool search(int from)
              boolean result = parentPattern.root.match(this, from, text);

            The acc_flags of the match method is 0, and what is found is java/util/regex/Patter$Node#match，
            Correct should use java/util/regex/Patter$Start#match
            */
            self.mir.method.is_abstract()
                || (self.mir.method.is_public() && !self.mir.method.is_final())
                || (self.mir.method.is_protected() && !self.mir.method.is_final())
                || (self.mir.method.acc_flags == 0)
        };
        trace!(
            "resolve_virtual_method resolve_again={}, acc_flags = {}",
            resolve_again,
            self.mir.method.acc_flags
        );
        if resolve_again {
            let this = self.args.get(0).unwrap();
            let this = util::oop::extract_ref(this);
            let this = this.read().unwrap();
            match &this.v {
                oop::RefKind::Inst(inst) => {
                    let cls = inst.class.read().unwrap();
                    let id = self.mir.method.get_id();
                    match cls.get_virtual_method(id.clone()) {
                        Ok(mir) => self.mir = mir,
                        _ => {
                            let cls = self.mir.method.class.read().unwrap();
                            warn!(
                                "resolve again failed, {}:{}, acc_flags = {}",
                                String::from_utf8_lossy(cls.name.as_slice()),
                                String::from_utf8_lossy(id.as_slice()),
                                self.mir.method.acc_flags
                            );
                        }
                    }
                }
                _ => (),
            };
        }
    }

    fn debug(&self) {
        let cls_name = { self.mir.method.class.read().unwrap().name.clone() };
        let name = self.mir.method.name.clone();
        let desc = self.mir.method.desc.clone();
        let cls_name = unsafe { std::str::from_utf8_unchecked(cls_name.as_slice()) };
        let name = unsafe { std::str::from_utf8_unchecked(name.as_slice()) };
        let desc = unsafe { std::str::from_utf8_unchecked(desc.as_slice()) };
        info!(
            "invoke method = {}:{}:{} static={} native={}",
            cls_name,
            name,
            desc,
            self.mir.method.is_static(),
            self.mir.method.is_native()
        );
    }
}

fn build_method_args(area: &DataAreaRef, sig: MethodSignature) -> Vec<Oop> {
    //Note: iter args by reverse, because of stack
    sig.args
        .iter()
        .rev()
        .map(|t| match t {
            SignatureType::Byte
            | SignatureType::Boolean
            | SignatureType::Int
            | SignatureType::Char
            | SignatureType::Short => {
                let mut area = area.borrow_mut();
                let v = area.stack.pop_int();
                Oop::new_int(v)
            }
            SignatureType::Long => {
                let mut area = area.borrow_mut();
                let v = area.stack.pop_long();
                Oop::new_long(v)
            }
            SignatureType::Float => {
                let mut area = area.borrow_mut();
                let v = area.stack.pop_float();
                Oop::new_float(v)
            }
            SignatureType::Double => {
                let mut area = area.borrow_mut();
                let v = area.stack.pop_double();
                Oop::new_double(v)
            }
            SignatureType::Object(_, _, _) | SignatureType::Array(_) => {
                let mut area = area.borrow_mut();
                area.stack.pop_ref()
            }
            t => unreachable!("t = {:?}", t),
        })
        .collect()
}

pub fn set_return(caller: Option<&DataAreaRef>, return_type: SignatureType, v: Option<Oop>) {
    match return_type {
        SignatureType::Byte
        | SignatureType::Short
        | SignatureType::Char
        | SignatureType::Int
        | SignatureType::Boolean => {
            let v = v.unwrap();
            let v = util::oop::extract_int(&v);
            let mut area = caller.unwrap().borrow_mut();
            area.stack.push_int(v);
        }
        SignatureType::Long => {
            let v = v.unwrap();
            let v = util::oop::extract_long(&v);
            let mut area = caller.unwrap().borrow_mut();
            area.stack.push_long(v);
        }
        SignatureType::Float => {
            let v = v.unwrap();
            let v = util::oop::extract_float(&v);
            let mut area = caller.unwrap().borrow_mut();
            area.stack.push_float(v);
        }
        SignatureType::Double => {
            let v = v.unwrap();
            let v = util::oop::extract_double(&v);
            let mut area = caller.unwrap().borrow_mut();
            area.stack.push_double(v);
        }
        SignatureType::Object(_, _, _) | SignatureType::Array(_) => {
            let v = v.unwrap();
            let mut area = caller.unwrap().borrow_mut();
            area.stack.push_ref(v);
        }
        SignatureType::Void => (),
    }
}
