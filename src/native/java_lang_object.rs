use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::OopRef;
use crate::runtime::JavaThread;
use std::sync::{Arc, Mutex};

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![new_fn(
        "registerNatives",
        "()V",
        Box::new(jvm_register_natives),
    )]
}

fn jvm_register_natives(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    Ok(None)
}
