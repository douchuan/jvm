#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{self, Oop};
use crate::runtime::{require_class3, JavaThread};
use crate::util;

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![new_fn(
        "newArray",
        "(Ljava/lang/Class;I)Ljava/lang/Object;",
        Box::new(jvm_newArray),
    )]
}

fn jvm_newArray(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    let mirror = args.get(0).unwrap();
    let component_cls = {
        let mirror = util::oop::extract_ref(mirror);
        let v = mirror.read().unwrap();
        match &v.v {
            oop::RefKind::Mirror(mirror) => mirror.target.clone().unwrap(),
            _ => unreachable!(),
        }
    };
    let length = args.get(1).unwrap();
    let length = util::oop::extract_int(length);

    //todo: throw NegativeArraySizeException
    let name = {
        let mut new_name = Vec::new();

        let cls = component_cls.read().unwrap();
        new_name.extend_from_slice("[".as_bytes());
        match cls.kind {
            oop::ClassKind::Instance(_) => {
                new_name.extend_from_slice("L".as_bytes());
                new_name.extend_from_slice(cls.name.as_slice());
                new_name.extend_from_slice(";".as_bytes());
            }
            oop::ClassKind::ObjectArray(_) => {
                new_name.extend_from_slice(cls.name.as_slice());
                new_name.extend_from_slice(";".as_bytes());
            }
            oop::ClassKind::TypeArray(_) => (),
        }

        new_name
    };

    let ary_cls = require_class3(None, name.as_slice()).unwrap();

    let v = Oop::new_ref_ary(ary_cls, length as usize);
    Ok(Some(v))
}
