use crate::native::{new_fn, JNIEnv, JNINativeMethod};
use crate::oop::OopRef;
use std::sync::{Arc, Mutex};

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![
        new_fn(
            "registerNatives",
            "()V",
            Box::new(jvm_register_natives),
        ),
        new_fn(
            "currentThread",
            "()Ljava/lang/Thread;",
            Box::new(jvm_current_thread)
        )
    ]
}

fn jvm_register_natives(env: JNIEnv, args: Vec<OopRef>) -> Option<OopRef> {
    None
}

fn jvm_current_thread(env: JNIEnv, args: Vec<OopRef>) -> Option<OopRef> {
    unimplemented!()
}