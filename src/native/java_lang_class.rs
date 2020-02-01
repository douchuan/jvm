use crate::native::{new_fn, JNIEnv, JNINativeMethod};
use crate::oop::{OopDesc, OopRef};
use std::sync::{Arc, Mutex};

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![
        new_fn(
            "registerNatives",
            "()V",
            Box::new(jvm_register_natives),
        ),
        new_fn(
            "desiredAssertionStatus0",
            "(Ljava/lang/Class;)Z",
            Box::new(jvm_desired_assertion_status0)
        )
    ]
}

fn jvm_register_natives(env: JNIEnv, args: Vec<OopRef>) -> Option<OopRef> {
    None
}

fn jvm_desired_assertion_status0(env: JNIEnv, args: Vec<OopRef>) -> Option<OopRef> {
    Some(OopDesc::new_int(0))
}