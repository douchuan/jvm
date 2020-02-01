use crate::native::{new_fn, JNIEnv, JNINativeMethod};
use crate::oop::OopRef;
use std::sync::{Arc, Mutex};

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![
        new_fn(
            "arraycopy",
            "(Ljava/lang/Object;ILjava/lang/Object;II)V",
            Box::new(jvm_arraycopy)),
        new_fn(
            "registerNatives",
            "()V",
            Box::new(jvm_register_natives),
        )
    ]
}

fn jvm_arraycopy(env: JNIEnv, args: Vec<OopRef>) -> Option<OopRef>{
    unimplemented!()
}

fn jvm_register_natives(env: JNIEnv, args: Vec<OopRef>) -> Option<OopRef> {
    None
}