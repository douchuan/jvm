#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{self, OopDesc, OopRef};
use crate::runtime::JavaThread;
use std::sync::{Arc, Mutex};

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![
        new_fn(
            "findSignal",
            "(Ljava/lang/String;)I",
            Box::new(jvm_findSignal),
        ),
        new_fn("handle0", "(IJ)J", Box::new(jvm_handle0)),
    ]
}

//todo: impl me
fn jvm_findSignal(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    Ok(Some(OopDesc::new_int(1)))
}

//todo: impl me
fn jvm_handle0(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    Ok(Some(OopDesc::new_long(0)))
}
