#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::Oop;

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![
        new_fn(
            "availableProcessors",
            "()I",
            Box::new(jvm_availableProcessors),
        ),
        new_fn("gc", "()V", Box::new(jvm_gc)),
    ]
}

//fixme:
fn jvm_availableProcessors(_env: JNIEnv, _args: &Vec<Oop>) -> JNIResult {
    Ok(Some(Oop::new_int(1)))
}

fn jvm_gc(_env: JNIEnv, _args: &Vec<Oop>) -> JNIResult {
    Ok(None)
}
