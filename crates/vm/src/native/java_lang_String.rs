#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::Oop;
use crate::types::JavaThreadRef;

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![new_fn(
        "intern",
        "()Ljava/lang/String;",
        Box::new(jvm_intern),
    )]
}

fn jvm_intern(_env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    let v = args.get(0).unwrap();
    Ok(Some(v.clone()))
}
