use crate::native;
use crate::oop::{self, Oop, ValueType};
use crate::runtime::{self, exception, frame::Frame, thread, DataArea, Interp};
use crate::types::{ClassRef, FrameRef, JavaThreadRef, MethodIdRef};
use crate::util;
use class_parser::MethodSignature;
use classfile::{consts as cls_const, BytesRef, SignatureType};
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

pub struct JavaCall {
    pub mir: MethodIdRef,
    pub args: Vec<Oop>,
    pub is_return_void: bool,
}

pub fn invoke_ctor(cls: ClassRef, desc: BytesRef, args: Vec<Oop>) {
    let ctor = {
        let cls = cls.get_class();
        cls.get_this_class_method(util::S_INIT.clone(), desc)
            .unwrap()
    };

    let mut jc = JavaCall::new_with_args(ctor, args);
    jc.invoke(None, false);
}

impl JavaCall {
    pub fn new_with_args(mir: MethodIdRef, args: Vec<Oop>) -> Self {
        let is_return_void = mir.method.signature.retype == SignatureType::Void;
        Self {
            mir,
            args,
            is_return_void,
        }
    }

    pub fn new(caller: &DataArea, mir: MethodIdRef) -> Result<JavaCall, ()> {
        let mut args = build_args_from_caller_stack(&caller, &mir.method.signature);

        //insert 'this' value
        let has_this = !mir.method.is_static();
        if has_this {
            let this = {
                let mut stack = caller.stack.borrow_mut();
                stack.pop_ref()
            };

            //check NPE
            if let Oop::Null = this {
                let cls_name = {
                    let cls = mir.method.class.get_class();
                    cls.name.clone()
                };

                error!("Java new failed, null this: {:?}", mir.method);

                //Fail fast, avoid a lot of logs, and it is not easy to locate the problem
                //                        panic!();

                let jt = runtime::thread::current_java_thread();
                let ex = exception::new(cls_const::J_NPE, None);
                let mut jt = jt.write().unwrap();
                jt.set_ex(ex);
                return Err(());
            }

            args.insert(0, this);
        }

        Ok(Self::new_with_args(mir, args))
    }
}

impl JavaCall {
    //the 'caller' for store return value
    pub fn invoke(&mut self, caller: Option<&DataArea>, force_no_resolve: bool) {
        /*
        Do resolve again first, because you can override in a native way such as:
        UnixFileSystem override FileSystem
            public abstract boolean checkAccess(File f, int access);

            public native boolean checkAccess(File f, int access);
        */
        self.resolve_virtual_method(force_no_resolve);
        self.debug();

        if self.mir.method.is_native() {
            self.invoke_native(caller);
        } else {
            self.invoke_java(caller);
        }

        let jt = runtime::thread::current_java_thread();
        let _ = jt.write().unwrap().frames.pop();
    }
}

impl JavaCall {
    fn invoke_java(&mut self, caller: Option<&DataArea>) {
        self.prepare_sync();

        let jt = runtime::thread::current_java_thread();
        match self.prepare_frame(false) {
            Ok(frame) => {
                {
                    jt.write().unwrap().frames.push(frame.clone());
                }

                let frame_h = frame.try_read().unwrap();
                let interp = Interp::new(frame_h);
                interp.run();

                //if return void, not need set return value
                if !self.is_return_void && !jt.read().unwrap().is_meet_ex() {
                    let return_value = {
                        let frame = frame.try_read().unwrap();
                        let area = frame.area.return_v.borrow();
                        area.clone()
                    };
                    set_return(caller, &self.mir.method.signature.retype, return_value);
                }
            }

            Err(ex) => {
                jt.write().unwrap().set_ex(ex);
            }
        }

        self.fin_sync();
    }

    fn invoke_native(&mut self, caller: Option<&DataArea>) {
        self.prepare_sync();

        let jt = runtime::thread::current_java_thread();
        let package = self.mir.method.class.get_class().name.as_slice();
        let desc = self.mir.method.desc.as_slice();
        let name = self.mir.method.name.as_slice();
        let method = native::find_symbol(package, name, desc);
        let v = match method {
            Some(method) => {
                let class = self.mir.method.class.clone();
                let env = native::new_jni_env(class);

                match self.prepare_frame(true) {
                    Ok(frame) => {
                        {
                            jt.write().unwrap().frames.push(frame);
                        }
                        method.invoke(env, &self.args)
                    }
                    Err(ex) => Err(ex),
                }
            }
            None => panic!(
                "Native method not found: {}:{}:{}",
                unsafe { std::str::from_utf8_unchecked(package) },
                unsafe { std::str::from_utf8_unchecked(name) },
                unsafe { std::str::from_utf8_unchecked(desc) },
            ),
        };

        match v {
            Ok(v) => {
                if !self.is_return_void && !jt.read().unwrap().is_meet_ex() {
                    set_return(caller, &self.mir.method.signature.retype, v);
                }
            }
            Err(ex) => jt.write().unwrap().set_ex(ex),
        }

        self.fin_sync();
    }

    fn prepare_sync(&mut self) {
        if self.mir.method.is_synchronized() {
            if self.mir.method.is_static() {
                let class = self.mir.method.class.get_class();
                class.monitor_enter();
            } else {
                let v = self.args.first().unwrap();
                let v = v.extract_ref();
                v.monitor_enter();
            }
        }
    }

    fn fin_sync(&mut self) {
        if self.mir.method.is_synchronized() {
            if self.mir.method.is_static() {
                let class = self.mir.method.class.get_class();
                class.monitor_exit();
            } else {
                let v = self.args.first().unwrap();
                let v = v.extract_ref();
                v.monitor_exit();
            }
        }
    }

    fn prepare_frame(&mut self, is_native: bool) -> Result<FrameRef, Oop> {
        let jt = runtime::thread::current_java_thread();
        let frame_len = { jt.read().unwrap().frames.len() };
        if frame_len >= runtime::consts::THREAD_MAX_STACK_FRAMES {
            let ex = exception::new(cls_const::J_SOE, None);
            return Err(ex);
        }

        let frame_id = frame_len + 1;
        let mut frame = Frame::new(self.mir.clone(), frame_id);

        if !is_native {
            //JVM spec, 2.6.1
            let mut local = frame.area.local.borrow_mut();
            let mut slot_pos: usize = 0;
            self.args.iter().for_each(|v| {
                let step = match v {
                    Oop::Int(v) => {
                        local.set_int(slot_pos, *v);
                        1
                    }
                    Oop::Float(v) => {
                        local.set_float(slot_pos, *v);
                        1
                    }
                    Oop::Double(v) => {
                        local.set_double(slot_pos, *v);
                        2
                    }
                    Oop::Long((v)) => {
                        local.set_long(slot_pos, *v);
                        2
                    }
                    _ => {
                        local.set_ref(slot_pos, v.clone());
                        1
                    }
                };

                slot_pos += step;
            });
        }

        let frame_ref = new_sync_ref!(frame);
        Ok(frame_ref)
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
            let rf = this.extract_ref();
            let ptr = rf.get_raw_ptr();
            unsafe {
                if let oop::RefKind::Inst(inst) = &(*ptr).v {
                    let name = self.mir.method.name.clone();
                    let desc = self.mir.method.desc.clone();
                    let cls = inst.class.get_class();
                    match cls.get_virtual_method(name.clone(), desc.clone()) {
                        Ok(mir) => self.mir = mir,
                        _ => {
                            let cls = self.mir.method.class.get_class();
                            warn!(
                                "resolve again failed, {}:{}:{}, acc_flags = {}",
                                String::from_utf8_lossy(cls.name.as_slice()),
                                String::from_utf8_lossy(name.as_slice()),
                                String::from_utf8_lossy(desc.as_slice()),
                                self.mir.method.acc_flags
                            );
                        }
                    }
                }
            }
        }
    }

    fn debug(&self) {
        info!(
            "invoke method = {:?}, static={} native={} sync={}",
            self.mir.method,
            self.mir.method.is_static(),
            self.mir.method.is_native(),
            self.mir.method.is_synchronized()
        );
    }
}

fn build_args_from_caller_stack(caller: &DataArea, sig: &MethodSignature) -> Vec<Oop> {
    let mut caller = caller.stack.borrow_mut();
    let mut args = Vec::new();

    //build args from caller's stack, so should rev the signature args
    for it in sig.args.iter().rev() {
        let v = match it {
            SignatureType::Byte
            | SignatureType::Boolean
            | SignatureType::Int
            | SignatureType::Char
            | SignatureType::Short => {
                let v = caller.pop_int();
                Oop::new_int(v)
            }
            SignatureType::Long => {
                let v = caller.pop_long();
                Oop::new_long(v)
            }
            SignatureType::Float => {
                let v = caller.pop_float();
                Oop::new_float(v)
            }
            SignatureType::Double => {
                let v = caller.pop_double();
                Oop::new_double(v)
            }
            SignatureType::Object(_, _, _) | SignatureType::Array(_) => caller.pop_ref(),
            t => unreachable!("t = {:?}", t),
        };

        args.push(v);
    }

    //the args built from caller's stack, should reverse args
    args.reverse();

    args
}

pub fn set_return(caller: Option<&DataArea>, return_type: &SignatureType, v: Option<Oop>) {
    match return_type {
        SignatureType::Byte
        | SignatureType::Short
        | SignatureType::Char
        | SignatureType::Int
        | SignatureType::Boolean => {
            let v = v.unwrap();
            let v = v.extract_int();
            let caller = caller.unwrap();
            let mut stack = caller.stack.borrow_mut();
            stack.push_int(v);
        }
        SignatureType::Long => {
            let v = v.unwrap();
            let v = v.extract_long();
            let caller = caller.unwrap();
            let mut stack = caller.stack.borrow_mut();
            stack.push_long(v);
        }
        SignatureType::Float => {
            let v = v.unwrap();
            let v = v.extract_float();
            let caller = caller.unwrap();
            let mut stack = caller.stack.borrow_mut();
            stack.push_float(v);
        }
        SignatureType::Double => {
            let v = v.unwrap();
            let v = v.extract_double();
            let caller = caller.unwrap();
            let mut stack = caller.stack.borrow_mut();
            stack.push_double(v);
        }
        SignatureType::Object(_, _, _) | SignatureType::Array(_) => {
            let v = v.unwrap();
            let caller = caller.unwrap();
            let mut stack = caller.stack.borrow_mut();
            stack.push_ref(v);
        }
        SignatureType::Void => (),
    }
}
