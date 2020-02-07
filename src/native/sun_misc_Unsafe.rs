#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{self, OopRef, OopDesc};
use crate::runtime::JavaThread;
use std::sync::{Arc, Mutex};

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![new_fn(
        "registerNatives",
        "()V",
        Box::new(jvm_registerNatives)),
         new_fn(
             "arrayBaseOffset",
             "(Ljava/lang/Class;)I",
             Box::new(jvm_arrayBaseOffset)),
    ]
}

fn jvm_registerNatives(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    Ok(None)
}

fn jvm_arrayBaseOffset(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    Ok(Some(OopDesc::new_int(0)))
}
