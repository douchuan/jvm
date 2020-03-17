#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::Oop;
use crate::runtime::JavaThread;

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![new_fn("VMSupportsCS8", "()Z", Box::new(jvm_VMSupportsCS8))]
}

fn jvm_VMSupportsCS8(_jt: &mut JavaThread, _env: JNIEnv, _args: Vec<Oop>) -> JNIResult {
    Ok(Some(Oop::new_int(0)))
}
