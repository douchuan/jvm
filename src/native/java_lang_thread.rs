use crate::native::{new_fn, JNIEnv, JNIResult, JNINativeMethod};
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
            Box::new(jvm_currentThread)
        ),
        new_fn(
            "setPriority0",
            "(I)V",
            Box::new(jvm_setPriority0)
        )
    ]
}

fn jvm_register_natives(env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    Ok(None)
}

fn jvm_currentThread(env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let r = env.lock().unwrap().java_thread_obj.clone();
    Ok(r)
}

fn jvm_setPriority0(env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    //todo: set native thread's priority
    Ok(None)
}