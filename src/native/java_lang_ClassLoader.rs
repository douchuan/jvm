#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop;
use crate::runtime::{self, JavaThread};
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
        new_fn(
            "findLoadedClass0",
            "(Ljava/lang/String;)Ljava/lang/Class;",
            Box::new(jvm_findLoadedClass0),
        ),
        new_fn(
            "findBootstrapClass",
            "(Ljava/lang/String;)Ljava/lang/Class;",
            Box::new(jvm_findBootstrapClass),
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

fn jvm_findLoadedClass0(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let name = args.get(1).unwrap();
    let name = util::oop::extract_str(name.clone());
    let name = name.replace(util::DOT_STR, util::PATH_SEP_STR);
    let v = match runtime::sys_dic_find(name.as_bytes()) {
        Some(cls) => {
            let cls = cls.lock().unwrap();
            cls.get_mirror()
        }
        None => oop::consts::get_null(),
    };
    Ok(Some(v))
}

fn jvm_findBootstrapClass(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    jvm_findLoadedClass0(_jt, _env, args)
}
