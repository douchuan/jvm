#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::Oop;

/// Stub for ReflectionFactory.<clinit> — bypasses JDK 9+ code that calls Set.of().
fn jvm_clinit(_env: JNIEnv, _args: &[Oop]) -> JNIResult {
    Ok(None)
}

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![new_fn("<clinit>", "()V", Box::new(jvm_clinit))]
}
