#![allow(non_snake_case)]

use crate::classfile::types::BytesRef;
use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{Oop, OopDesc, OopRef};
use crate::runtime::JavaCall;
use crate::runtime::{self, JavaThread};
use crate::util;
use std::sync::{Arc, Mutex};

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![new_fn(
        "registerNatives",
        "()V",
        Box::new(jvm_registerNatives),
    )]
}

fn jvm_registerNatives(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    Ok(None)
}
