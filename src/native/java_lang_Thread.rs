#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{self, OopDesc, OopRef};
use crate::runtime::{JavaCall, JavaThread, Stack};
use crate::util::new_method_id;
use std::sync::{Arc, Mutex};

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![
        new_fn("registerNatives", "()V", Box::new(jvm_register_natives)),
        new_fn(
            "currentThread",
            "()Ljava/lang/Thread;",
            Box::new(jvm_currentThread),
        ),
        new_fn("setPriority0", "(I)V", Box::new(jvm_setPriority0)),
        new_fn("isAlive", "()Z", Box::new(jvm_isAlive)),
        new_fn("start0", "()V", Box::new(jvm_start0)),
    ]
}

fn jvm_register_natives(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    Ok(None)
}

fn jvm_currentThread(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let r = env.lock().unwrap().java_thread_obj.clone();
    Ok(r)
}

fn jvm_setPriority0(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    //todo: set native thread's priority
    Ok(None)
}

fn jvm_isAlive(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    //todo: impl
    Ok(Some(OopDesc::new_int(0)))
}

fn jvm_start0(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let thread_oop = args.get(0).unwrap();
    let cls = {
        let v = thread_oop.lock().unwrap();
        match &v.v {
            oop::Oop::Inst(inst) => inst.class.clone(),
            _ => unreachable!(),
        }
    };

    let name = {
        let cls = cls.lock().unwrap();
        cls.name.clone()
    };

    if String::from_utf8_lossy(name.as_slice()) == "java/lang/ref/Reference$ReferenceHandler" {
        Ok(None)
    } else {
        //todo: impl threads manager

        let mir = {
            let cls = cls.lock().unwrap();
            let id = new_method_id(b"run", b"()V");
            cls.get_virtual_method(id).unwrap()
        };

        let mut jt = JavaThread::new();
        let mut stack = Stack::new(0);
        let args = vec![thread_oop.clone()];
        let mut jc = JavaCall::new_with_args(&mut jt, mir, args);
        jc.invoke(&mut jt, &mut stack, false);

        Ok(None)
    }
}
