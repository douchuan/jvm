#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::Oop;

/// Stub for ObjectStreamClass.<clinit> — bypasses JDK 9+ serialization
/// infrastructure that calls Set.of()/Map.of() which we don't implement.
fn jvm_clinit(_env: JNIEnv, _args: &[Oop]) -> JNIResult {
    Ok(None)
}

fn jvm_initNative(_env: JNIEnv, _args: &[Oop]) -> JNIResult {
    Ok(None)
}

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![
        new_fn("<clinit>", "()V", Box::new(jvm_clinit)),
        new_fn("initNative", "()V", Box::new(jvm_initNative)),
    ]
}
