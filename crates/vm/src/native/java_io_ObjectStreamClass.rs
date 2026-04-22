#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::Oop;

fn jvm_initNative(_env: JNIEnv, _args: &[Oop]) -> JNIResult {
    Ok(None)
}

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![new_fn("initNative", "()V", Box::new(jvm_initNative))]
}
