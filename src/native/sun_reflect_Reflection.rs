#![allow(non_snake_case)]

use crate::native::{self, new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::Oop;
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

fn jvm_getCallerClass(jt: &mut JavaThread, _env: JNIEnv, _args: Vec<Oop>) -> JNIResult {
    let mut callers = jt.callers.clone();

    let cur = callers.pop().unwrap(); //pop cur method
    assert_eq!(cur.method.name.as_slice(), b"getCallerClass");

    loop {
        let caller = callers.pop().unwrap();
        if caller
            .method
            .check_annotation(b"Lsun/reflect/CallerSensitive;")
        {
            continue;
        }

        let cls = caller.method.class.lock().unwrap();
        //        error!("getCallerClass name = {}", String::from_utf8_lossy(cls.name.as_slice()));
        return Ok(Some(cls.get_mirror()));
    }
}

fn jvm_getClassAccessFlags(jt: &mut JavaThread, env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    native::java_lang_Class::jvm_getModifiers(jt, env, args)
}
