#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{Oop, OopDesc};
use crate::runtime::JavaThread;
use crate::types::OopRef;

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![
        new_fn(
            "doubleToRawLongBits",
            "(D)J",
            Box::new(jvm_doubleToRawLongBits),
        ),
        new_fn("longBitsToDouble", "(J)D", Box::new(jvm_longBitsToDouble)),
    ]
}

fn jvm_doubleToRawLongBits(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let d = args.get(0).unwrap();
    let d = d.lock().unwrap();
    match d.v {
        Oop::Double(v) => {
            let v = v.to_bits().to_be_bytes();
            let v = i64::from_be_bytes([v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]]);
            Ok(Some(OopDesc::new_long(v)))
        }
        _ => unreachable!(),
    }
}

fn jvm_longBitsToDouble(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let l = args.get(0).unwrap();
    let v = l.lock().unwrap();
    match v.v {
        Oop::Long(v) => {
            let v = v.to_be_bytes();
            let v = f64::from_be_bytes([v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]]);
            Ok(Some(OopDesc::new_double(v)))
        }
        _ => unreachable!(),
    }
}
