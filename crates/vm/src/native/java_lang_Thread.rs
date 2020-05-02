#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{self, Oop};
use crate::runtime::{self, JavaCall, JavaThread};
use crate::util;
use crate::types::JavaThreadRef;

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![
        new_fn("registerNatives", "()V", Box::new(jvm_registerNatives)),
        new_fn(
            "currentThread",
            "()Ljava/lang/Thread;",
            Box::new(jvm_currentThread),
        ),
        new_fn("setPriority0", "(I)V", Box::new(jvm_setPriority0)),
        new_fn("isAlive", "()Z", Box::new(jvm_isAlive)),
        new_fn("start0", "()V", Box::new(jvm_start0)),
        new_fn("isInterrupted", "(Z)Z", Box::new(jvm_isInterrupted)),
    ]
}

fn jvm_registerNatives(_jt: JavaThreadRef, _env: JNIEnv, _args: Vec<Oop>) -> JNIResult {
    Ok(None)
}

fn jvm_currentThread(_jt: JavaThreadRef, env: JNIEnv, _args: Vec<Oop>) -> JNIResult {
    let r = env.read().unwrap().java_thread_obj.clone();
    Ok(r)
}

fn jvm_setPriority0(_jt: JavaThreadRef, _env: JNIEnv, _args: Vec<Oop>) -> JNIResult {
    //todo: set native thread's priority
    Ok(None)
}

fn jvm_isAlive(_jt: JavaThreadRef, _env: JNIEnv, _args: Vec<Oop>) -> JNIResult {
    //todo: impl
    Ok(Some(Oop::new_int(0)))
}

fn jvm_start0(_jt: JavaThreadRef, _env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    let thread_oop = args.get(0).unwrap();
    let cls = {
        let thread_oop = util::oop::extract_ref(thread_oop);
        let v = thread_oop.read().unwrap();
        match &v.v {
            oop::RefKind::Inst(inst) => inst.class.clone(),
            _ => unreachable!(),
        }
    };

    let name = {
        let cls = cls.read().unwrap();
        cls.name.clone()
    };

    if name.as_slice() == "java/lang/ref/Reference$ReferenceHandler".as_bytes() {
        Ok(None)
    } else {
        //todo: impl threads manager

        let mir = {
            let cls = cls.read().unwrap();
            cls.get_virtual_method(b"run", b"()V").unwrap()
        };

        let jt = JavaThread::new();
        let args = vec![thread_oop.clone()];
        let mut jc = JavaCall::new_with_args(jt.clone(), mir, args);
        let area = runtime::DataArea::new(0, 0);
        jc.invoke(jt, Some(&area), false);

        Ok(None)
    }
}

fn jvm_isInterrupted(_jt: JavaThreadRef, _env: JNIEnv, _args: Vec<Oop>) -> JNIResult {
    //todo: fix me
    let v = Oop::new_int(0);
    Ok(Some(v))
}
