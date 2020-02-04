#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{self, OopRef};
use crate::runtime::JavaThread;
use std::sync::{Arc, Mutex};

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![
        new_fn(
            "doPrivileged",
            "(Ljava/security/PrivilegedAction;)Ljava/lang/Object;",
            Box::new(jvm_doPrivileged),
        ),
        new_fn(
            "getStackAccessControlContext",
            "()Ljava/security/AccessControlContext;",
            Box::new(jvm_getStackAccessControlContext),
        ),
    ]
}

fn jvm_doPrivileged(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    unimplemented!()
}

fn jvm_getStackAccessControlContext(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    Ok(Some(oop::consts::get_null()))
}
