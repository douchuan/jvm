#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{Oop, OopRef, OopDesc};
use crate::runtime::JavaThread;
use std::sync::{Arc, Mutex};

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![
        new_fn(
        "doubleToRawLongBits",
        "(D)J",
        Box::new(jvm_doubleToRawLongBits)),

        new_fn(
            "longBitsToDouble",
            "(J)D",
            Box::new(jvm_longBitsToDouble)),
    ]
}

fn jvm_doubleToRawLongBits(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    match args.get(0) {
        Some(v) => {
            let v = v.lock().unwrap();
            match v.v {
                Oop::Double(v) => {
                    let v = v.to_bits().to_be_bytes();
                    let v = i64::from_be_bytes([v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]]);
                    Ok(Some(OopDesc::new_long(v)))
                }
                _ => unreachable!()
            }
        }
        _ => unreachable!()
    }
}

fn jvm_longBitsToDouble(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    match args.get(0) {
        Some(v) => {
            let v = v.lock().unwrap();
            match v.v {
                Oop::Long(v) => {
                    let v = v.to_be_bytes();
                    let v = f64::from_be_bytes([v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]]);
                    Ok(Some(OopDesc::new_double(v)))
                }
                _ => unreachable!()
            }
        }
        _ => unreachable!()
    }
}
