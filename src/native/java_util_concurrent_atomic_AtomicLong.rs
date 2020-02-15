#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::OopDesc;
use crate::runtime::JavaThread;
use crate::types::OopRef;

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![new_fn("VMSupportsCS8", "()Z", Box::new(jvm_VMSupportsCS8))]
}

fn jvm_VMSupportsCS8(_jt: &mut JavaThread, _env: JNIEnv, _args: Vec<OopRef>) -> JNIResult {
    Ok(Some(OopDesc::new_int(0)))
}
