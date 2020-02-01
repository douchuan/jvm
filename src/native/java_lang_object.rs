use crate::native::{new_fn, JNIEnv, JNIResult, JNINativeMethod};
use crate::oop::OopRef;
use std::sync::{Arc, Mutex};

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![
        new_fn(
            "registerNatives",
            "()V",
            Box::new(jvm_register_natives),
        )
    ]
}

fn jvm_register_natives(env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    Ok(None)
}