#![allow(non_snake_case)]

use crate::classfile::types::BytesRef;
use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{Oop, OopDesc, OopRef};
use crate::runtime::JavaCall;
use crate::runtime::{self, JavaThread};
use crate::util;
use std::sync::{Arc, Mutex};

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![
        new_fn("registerNatives", "()V", Box::new(jvm_registerNatives)),
        new_fn(
            "findBuiltinLib",
            "(Ljava/lang/String;)Ljava/lang/String;",
            Box::new(jvm_findBuiltinLib),
        ),
    ]
}

fn jvm_registerNatives(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    Ok(None)
}

fn jvm_findBuiltinLib(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let name = args.get(0).unwrap();
    let name = util::oop::extract_str(name.clone());
    trace!("name = {}", String::from_utf8_lossy(name.as_slice()));
    Ok(None)
}
