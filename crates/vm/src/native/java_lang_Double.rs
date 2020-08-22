#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::Oop;

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

fn jvm_doubleToRawLongBits(_env: JNIEnv, args: &[Oop]) -> JNIResult {
    let v = args.get(0).unwrap().extract_double();
    let v = v.to_bits().to_be_bytes();
    let v = i64::from_be_bytes(v);
    Ok(Some(Oop::new_long(v)))
}

fn jvm_longBitsToDouble(_env: JNIEnv, args: &[Oop]) -> JNIResult {
    let v = args.get(0).unwrap().extract_long();
    let v = v.to_be_bytes();
    let v = f64::from_be_bytes(v);
    Ok(Some(Oop::new_double(v)))
}
