#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::Oop;

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![
        new_fn("initIDs", "()V", Box::new(jvm_initIDs)),
        // JDK 9+: native methods for fd/handle mapping
        new_fn("getHandle", "(I)J", Box::new(jvm_getHandle)),
        new_fn("setHandle", "(J)V", Box::new(jvm_setHandle)),
        new_fn("close0", "()V", Box::new(jvm_close0)),
        new_fn("getAppend", "(I)Z", Box::new(jvm_getAppend)),
    ]
}

fn jvm_initIDs(_env: JNIEnv, _args: &[Oop]) -> JNIResult {
    Ok(None)
}

fn jvm_getHandle(_env: JNIEnv, args: &[Oop]) -> JNIResult {
    // Static method: args[0] is the int fd parameter
    let fd = args.get(0).unwrap().extract_int();
    Ok(Some(Oop::new_long(fd as i64)))
}

fn jvm_setHandle(_env: JNIEnv, args: &[Oop]) -> JNIResult {
    // No-op: we don't use native file handles
    Ok(None)
}

fn jvm_close0(_env: JNIEnv, _args: &[Oop]) -> JNIResult {
    // No-op: we don't track native file handles
    Ok(None)
}

fn jvm_getAppend(_env: JNIEnv, _args: &[Oop]) -> JNIResult {
    // Return false (not append mode)
    Ok(Some(Oop::new_int(0)))
}
