use crate::native::{new_fn, JNIEnv, JNIResult, JNINativeMethod};
use crate::oop::{self, OopRef};
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
        )
    ]
}

fn jvm_doPrivileged(env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    unimplemented!()
}

fn jvm_getStackAccessControlContext(env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    Ok(Some(oop::consts::get_null()))
}

