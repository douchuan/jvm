use crate::native::{new_fn, JNIEnv, JNINativeMethod};
use crate::oop::OopRef;
use std::sync::{Arc, Mutex};

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![
        new_fn(
            "doPrivileged",
            "(Ljava/security/PrivilegedAction;)Ljava/lang/Object;",
            Box::new(jvm_do_privileged),
        )
    ]
}

fn jvm_do_privileged(env: JNIEnv, args: Vec<OopRef>) -> Option<OopRef> {
    unimplemented!()
}

