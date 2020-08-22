#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::Oop;

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![
        new_fn(
            "findSignal",
            "(Ljava/lang/String;)I",
            Box::new(jvm_findSignal),
        ),
        new_fn("handle0", "(IJ)J", Box::new(jvm_handle0)),
    ]
}

//todo: impl me
fn jvm_findSignal(_env: JNIEnv, _args: &[Oop]) -> JNIResult {
    Ok(Some(Oop::new_int(1)))
}

//todo: impl me
fn jvm_handle0(_env: JNIEnv, _args: &[Oop]) -> JNIResult {
    Ok(Some(Oop::new_long(0)))
}
