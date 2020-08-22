#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{self, Oop};

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![new_fn(
        "getLookupCacheURLs",
        "(Ljava/lang/ClassLoader;)[Ljava/net/URL;",
        Box::new(jvm_getLookupCacheURLs),
    )]
}

fn jvm_getLookupCacheURLs(_env: JNIEnv, _args: &[Oop]) -> JNIResult {
    Ok(Some(oop::consts::get_null()))
}
