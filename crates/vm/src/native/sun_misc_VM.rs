#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::Oop;
use crate::runtime::JavaThread;

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![new_fn("initialize", "()V", Box::new(jvm_initialize))]
}

fn jvm_initialize(_jt: &mut JavaThread, _env: JNIEnv, _args: Vec<Oop>) -> JNIResult {
    Ok(None)
}
