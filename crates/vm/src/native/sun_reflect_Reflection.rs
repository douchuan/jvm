#![allow(non_snake_case)]

use crate::native::{self, new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::Oop;
use crate::runtime;

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

fn jvm_getCallerClass(_env: JNIEnv, _args: &[Oop]) -> JNIResult {
    let jt = runtime::thread::current_java_thread();
    let mut callers = { jt.read().unwrap().frames.clone() };

    {
        let cur = {
            let cur = callers.pop().unwrap(); //pop cur method
            let cur = cur.try_read().unwrap();
            cur.mir.clone()
        };
        debug_assert_eq!(cur.method.name.as_slice(), b"getCallerClass");
    }

    loop {
        let caller = {
            let caller = callers.pop().unwrap();
            let caller = caller.try_read().unwrap();
            caller.mir.clone()
        };
        if caller
            .method
            .check_annotation(b"Lsun/reflect/CallerSensitive;")
        {
            continue;
        }

        let cls = caller.method.class.get_class();
        //        error!("getCallerClass name = {}", String::from_utf8_lossy(cls.name.as_slice()));
        return Ok(Some(cls.get_mirror()));
    }
}

fn jvm_getClassAccessFlags(env: JNIEnv, args: &[Oop]) -> JNIResult {
    native::java_lang_Class::jvm_getModifiers(env, args)
}
