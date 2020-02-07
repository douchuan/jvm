#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{self, OopRef};
use crate::runtime::JavaThread;
use std::sync::{Arc, Mutex};

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![new_fn("getCallerClass", "()Ljava/lang/Class;", Box::new(jvm_getCallerClass))]
}

fn jvm_getCallerClass(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    //todo: impl
    Ok(Some(oop::consts::get_null()))
}