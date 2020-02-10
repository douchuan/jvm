#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{OopDesc, OopRef};
use crate::runtime::JavaThread;

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![
        new_fn("registerNatives", "()V", Box::new(jvm_registerNatives)),
        new_fn("hashCode", "()I", Box::new(jvm_hashCode)),
    ]
}

fn jvm_registerNatives(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    Ok(None)
}

fn jvm_hashCode(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let v = args.get(0).unwrap();
    let v = v.lock().unwrap().hash_code;
    trace!("hashCode = {}", v);
    Ok(Some(OopDesc::new_int(v as i32)))
}
