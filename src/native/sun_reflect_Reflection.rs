#![allow(non_snake_case)]

use crate::native::{self, new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::OopRef;
use crate::runtime::JavaThread;

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![
        new_fn(
            "getCallerClass",
            "()Ljava/lang/Class;",
            Box::new(jvm_getCallerClass),
        ),
        new_fn(
            "getClassAccessFlags",
            "(Ljava/lang/Class;)I",
            Box::new(jvm_getClassAccessFlags),
        ),
    ]
}

fn jvm_getCallerClass(jt: &mut JavaThread, _env: JNIEnv, _args: Vec<OopRef>) -> JNIResult {
    let mut callers = jt.callers.clone();

    callers.pop(); //pop cur method

    loop {
        let caller = callers.pop().unwrap();
        if caller
            .method
            .check_annotation(b"Lsun/reflect/CallerSensitive;")
        {
            continue;
        }

        let cls = caller.method.class.lock().unwrap();
        return Ok(Some(cls.get_mirror()));
    }
}

fn jvm_getClassAccessFlags(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    native::java_lang_Class::jvm_getModifiers(jt, env, args)
}
