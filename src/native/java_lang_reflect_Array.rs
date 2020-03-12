#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{self, Oop, ValueType};
use crate::runtime::{require_class3, JavaThread};
use crate::types::ClassRef;
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
    //todo: throw NegativeArraySizeException
    let length = args.get(1).unwrap();
    let length = util::oop::extract_int(length);

    let (vt, component_cls) = {
        let mirror = util::oop::extract_ref(mirror);
        let v = mirror.read().unwrap();
        match &v.v {
            oop::RefKind::Mirror(mirror) => (mirror.value_type, mirror.target.clone()),
            _ => unreachable!(),
        }
    };
    let name = build_ary_name(vt, component_cls);
    let ary_cls = require_class3(None, name.as_slice()).unwrap();
    let v = Oop::new_ref_ary(ary_cls, length as usize);

    Ok(Some(v))
}

fn build_ary_name(vt: ValueType, component_cls: Option<ClassRef>) -> Vec<u8> {
    let mut name = Vec::from("[");

    match vt {
        ValueType::BYTE
        | ValueType::BOOLEAN
        | ValueType::CHAR
        | ValueType::SHORT
        | ValueType::INT
        | ValueType::LONG
        | ValueType::FLOAT
        | ValueType::DOUBLE => name.extend_from_slice(vt.into()),
        ValueType::OBJECT | ValueType::ARRAY => {
            let cls = component_cls.unwrap();
            let cls = cls.read().unwrap();
            match cls.kind {
                oop::ClassKind::Instance(_) => {
                    name.extend_from_slice("L".as_bytes());
                    name.extend_from_slice(cls.name.as_slice());
                    name.extend_from_slice(";".as_bytes());
                }
                oop::ClassKind::ObjectArray(_) => {
                    name.extend_from_slice(cls.name.as_slice());
                    name.extend_from_slice(";".as_bytes());
                }
                oop::ClassKind::TypeArray(_) => unimplemented!(),
            }
        }

        ValueType::VOID => unreachable!(),
    }

    name
}
