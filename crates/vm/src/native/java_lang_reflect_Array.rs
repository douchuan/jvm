#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{self, Oop, ValueType};
use crate::runtime::require_class3;
use crate::types::ClassRef;

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![
        new_fn(
            "newArray",
            "(Ljava/lang/Class;I)Ljava/lang/Object;",
            Box::new(jvm_newArray),
        ),
        new_fn(
            "getLength",
            "(Ljava/lang/Object;)I",
            Box::new(jvm_getLength),
        ),
    ]
}

fn jvm_newArray(_env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    let mirror = args.get(0).unwrap();
    //todo: throw NegativeArraySizeException
    let length = args.get(1).unwrap().extract_int();

    let (vt, component_cls) = {
        let rf = mirror.extract_ref();
        let mirror = rf.extract_mirror();
        (mirror.value_type, mirror.target.clone())
    };
    let name = build_ary_name(vt, component_cls);
    let ary_cls = require_class3(None, name.as_slice()).unwrap();
    let v = Oop::new_ref_ary(ary_cls, length as usize);

    Ok(Some(v))
}

fn jvm_getLength(_env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    let ary = args.get(0).unwrap();
    let rf = ary.extract_ref();
    let ptr = rf.get_raw_ptr();
    let len = unsafe {
        match &(*ptr).v {
            oop::RefKind::TypeArray(ary) => ary.len(),
            oop::RefKind::Array(ary) => ary.elements.len(),
            _ => unreachable!(),
        }
    };

    let v = Oop::new_int(len as i32);
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
            let cls = cls.get_class();
            match cls.get_class_kind_type() {
                oop::class::ClassKindType::Instance => {
                    name.extend_from_slice(b"L");
                    name.extend_from_slice(cls.name.as_slice());
                    name.extend_from_slice(b";");
                }
                oop::class::ClassKindType::ObjectAry => {
                    name.extend_from_slice(cls.name.as_slice());
                    name.extend_from_slice(b";");
                }
                oop::class::ClassKindType::TypAry => unimplemented!(),
            }
        }

        ValueType::VOID => unreachable!(),
    }

    name
}
