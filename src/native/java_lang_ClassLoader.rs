#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::runtime::JavaThread;
use crate::types::OopRef;
use crate::util;

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![
        new_fn("registerNatives", "()V", Box::new(jvm_registerNatives)),
        new_fn(
            "findBuiltinLib",
            "(Ljava/lang/String;)Ljava/lang/String;",
            Box::new(jvm_findBuiltinLib),
        ),
    ]
}

fn jvm_registerNatives(_jt: &mut JavaThread, _env: JNIEnv, _args: Vec<OopRef>) -> JNIResult {
    Ok(None)
}

fn jvm_findBuiltinLib(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let name = args.get(0).unwrap();
    let name = util::oop::extract_str(name.clone());
    trace!("name = {}", name);
    Ok(None)
}
