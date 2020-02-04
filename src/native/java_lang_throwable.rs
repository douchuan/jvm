#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::OopRef;
use crate::runtime::JavaThread;
use std::sync::{Arc, Mutex};

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![
        new_fn(
            "fillInStackTrace",
            "(I)Ljava/lang/Throwable;",
            Box::new(jvm_fillInStackTrace),
        ),
    ]
}

fn jvm_fillInStackTrace(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let r = env.lock().unwrap().java_thread_obj.clone();
    Ok(r)
}

