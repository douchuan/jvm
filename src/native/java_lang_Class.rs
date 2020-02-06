#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{Oop, OopDesc, OopRef, ValueType};
use crate::runtime::{require_class3, JavaThread};
use crate::util;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![
        new_fn("registerNatives", "()V", Box::new(jvm_register_natives)),
        new_fn(
            "desiredAssertionStatus0",
            "(Ljava/lang/Class;)Z",
            Box::new(jvm_desiredAssertionStatus0),
        ),
        new_fn(
            "getPrimitiveClass",
            "(Ljava/lang/String;)Ljava/lang/Class;",
            Box::new(jvm_getPrimitiveClass),
        ),
    ]
}

lazy_static! {
    static ref PRIM_MIRROS: Mutex<HashMap<String, OopRef>> = {
        let hm = HashMap::new();
        Mutex::new(hm)
    };
    static ref SIGNATURE_DIC: HashMap<&'static str, &'static str> = {
        let dic: HashMap<&'static str, &'static str> = [
            ("byte", "B"),
            ("boolean", "Z"),
            ("char", "C"),
            ("short", "S"),
            ("int", "I"),
            ("float", "F"),
            ("long", "J"),
            ("double", "D"),
            ("void", "V"),
        ]
        .iter()
        .cloned()
        .collect();

        dic
    };
}

pub fn init() {
    lazy_static::initialize(&PRIM_MIRROS);

    [
        "I", "Z", "B", "C", "S", "F", "J", "D", "V", "[I", "[Z", "[B", "[C", "[S", "[F", "[J", "[D",
    ]
    .iter()
    .for_each(|&t| {
        let is_prim_ary = t.as_bytes()[0] == b'[';
        let vt = if is_prim_ary {
            ValueType::from(&t.as_bytes()[1])
        } else {
            ValueType::from(&t.as_bytes()[0])
        };

        let mirror = OopDesc::new_prim_mirror(vt);
        if is_prim_ary {
            let target = require_class3(None, t.as_bytes()).unwrap();
            let mut mirror = mirror.lock().unwrap();
            match &mut mirror.v {
                Oop::Mirror(mirror) => {
                    mirror.target = Some(target);
                }
                _ => unreachable!(),
            }
        }

        util::sync_call_ctx(&PRIM_MIRROS, |mirrors| {
            mirrors.insert(t.to_string(), mirror);
        });
    });
}

fn jvm_register_natives(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    Ok(None)
}

fn jvm_desiredAssertionStatus0(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    Ok(Some(OopDesc::new_int(0)))
}

fn jvm_getPrimitiveClass(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let v = args.get(0).unwrap();

    let v = {
        let v = v.lock().unwrap();
        match &v.v {
            Oop::Str(s) => s.clone(),
            _ => unreachable!(),
        }
    };

    let s = std::str::from_utf8(v.as_slice()).unwrap();
    match SIGNATURE_DIC.get(s) {
        Some(&s) => {
            //todo: avoid mutex lock, it's only read
            util::sync_call(&PRIM_MIRROS, |mirros| {
                Ok(mirros.get(s).map(|it| it.clone()))
            })
        }
        _ => unreachable!("Unknown primitive type: {}", s),
    }
}
