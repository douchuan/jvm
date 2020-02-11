#![allow(non_snake_case)]

use crate::native::{self, new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{self, Oop, OopDesc, OopRef};
use crate::runtime::{self, JavaThread};
use crate::util;
use std::sync::{Arc, Mutex};

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![new_fn(
        "newInstance0",
        "(Ljava/lang/reflect/Constructor;[Ljava/lang/Object;)Ljava/lang/Object;",
        Box::new(jvm_newInstance0),
    )]
}

fn jvm_newInstance0(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let ctor = args.get(0).unwrap();
    let arguments = args.get(1).unwrap();

    let clazz = runtime::reflect::get_Constructor_clazz(ctor.clone());
    let target_cls = {
        let v = clazz.lock().unwrap();
        match &v.v {
            Oop::Mirror(mirror) => mirror.target.clone().unwrap(),
            _ => unreachable!(),
        }
    };

    let signature = runtime::reflect::get_Constructor_signature(ctor.clone());

    let mut ctor_args = Vec::new();
    {
        let v = arguments.lock().unwrap();
        match &v.v {
            Oop::Array(ary) => {
                ctor_args.extend_from_slice(ary.elements.as_slice());
            }
            Oop::Null => ctor_args.push(oop::consts::get_null()),
            t => unreachable!("t = {:?}", t),
        }
    }

    let oop = OopDesc::new_inst(target_cls.clone());
    ctor_args.insert(0, oop.clone());

    runtime::java_call::invoke_ctor(jt, target_cls, signature.as_slice(), ctor_args);

    Ok(Some(oop))
}
