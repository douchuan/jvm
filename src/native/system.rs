use crate::native::{new_fn, JNIEnv, JNINativeMethod};
use crate::oop::OopRef;
use std::sync::{Arc, Mutex};

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![
        new_fn(
            b"arraycopy",
            b"(Ljava/lang/Object;ILjava/lang/Object;II)V",
            Box::new(jvm_arraycopy)),
        new_fn(
            b"registerNatives",
            b"()V",
            Box::new(jvm_register_natives),
        )
    ]
}

fn jvm_arraycopy(env: JNIEnv, args: Vec<OopRef>) -> Option<OopRef>{
    trace!("jvm_arraycopy called");
    None
}

fn jvm_register_natives(env: JNIEnv, args: Vec<OopRef>) -> Option<OopRef> {
    trace!("jvm_register_natives called");
    None
}