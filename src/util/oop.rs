use crate::classfile::types::BytesRef;
use crate::oop::{Oop, OopRef};
use crate::runtime::require_class3;
use std::sync::{Arc};

pub fn is_str(v: OopRef) -> bool {
    let v = v.lock().unwrap();
    match &v.v {
        Oop::Str(s) => true,
        Oop::Inst(inst) => {
            let cls = inst.class.lock().unwrap();
            warn!(
                "is_str_oop name = {}",
                String::from_utf8_lossy(cls.name.as_slice())
            );
            cls.name.as_slice() == b"java/lang/String"
        }
        _ => false,
    }
}

pub fn extract_str(v: OopRef) -> BytesRef {
    {
        let v = v.lock().unwrap();
        match &v.v {
            Oop::Str(s) => return s.clone(),
            _ => (),
        }
    }

    let fid = {
        let v = v.lock().unwrap();
        match &v.v {
            Oop::Inst(inst) => {
                let cls = inst.class.lock().unwrap();
                cls.get_field_id(b"value", b"[C", false)
            }
            _ => unreachable!(),
        }
    };

    let cls_string = require_class3(None, b"java/lang/String").unwrap();
    let value_ary = {
        let cls = cls_string.lock().unwrap();
        cls.get_field_value(v.clone(), fid)
    };

    let value_ary = value_ary.lock().unwrap();
    let ary: Vec<u8> = match &value_ary.v {
        Oop::Array(ary) => ary
            .elements
            .iter()
            .map(|it| {
                let v = it.lock().unwrap();
                match &v.v {
                    Oop::Int(v) => *v as u8,
                    _ => unreachable!(),
                }
            })
            .collect(),
        _ => unreachable!(),
    };

    new_ref!(ary)
}

pub fn if_acmpeq(v1: OopRef, v2: OopRef) -> bool {
    if Arc::ptr_eq(&v1, &v2) {
        true
    } else {
        if is_str(v2.clone()) && is_str(v1.clone()) {
            let v2 = extract_str(v2.clone());
            let v1 = extract_str(v1.clone());
            if v2.as_slice() == v1.as_slice() {
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}
