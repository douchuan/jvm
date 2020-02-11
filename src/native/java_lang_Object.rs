#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{self, Oop, OopDesc, OopRef};
use crate::runtime::{require_class3, JavaThread};

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![
        new_fn("registerNatives", "()V", Box::new(jvm_registerNatives)),
        new_fn("hashCode", "()I", Box::new(jvm_hashCode)),
        new_fn("clone", "()Ljava/lang/Object;", Box::new(jvm_clone)),
    ]
}

fn jvm_registerNatives(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    Ok(None)
}

fn jvm_hashCode(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let v = args.get(0).unwrap();
    let v = v.lock().unwrap().hash_code;
    trace!("hashCode = {}", v);
    Ok(Some(OopDesc::new_int(v as i32)))
}

fn jvm_clone(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    //    let java_lang_Cloneable = require_class3(None, b"java/lang/Cloneable").unwrap();
    let this_obj = args.get(0).unwrap();
    Ok(Some(this_obj.clone()))
}
