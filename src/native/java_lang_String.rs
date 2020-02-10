#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{self, OopDesc, OopRef};
use crate::runtime::JavaThread;
use std::sync::{Arc, Mutex};

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![new_fn(
        "intern",
        "()Ljava/lang/String;",
        Box::new(jvm_intern),
    )]
}

fn jvm_intern(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let v = {
        match args.get(0) {
            Some(v) => {
                {
                    let v = v.lock().unwrap();
                    match &v.v {
                        oop::Oop::Str(s) => (),
                        _ => unimplemented!(),
                    }
                }

                v.clone()
            }
            _ => unreachable!(),
        }
    };

    //    warn!("jvm_intern s = {}", String::from_utf8_lossy(s.as_slice()));

    Ok(Some(v))
}
