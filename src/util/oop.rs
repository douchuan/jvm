use crate::oop::{self, Oop};
use crate::runtime::{self, require_class3, JavaThread};
use crate::types::OopRef;
use std::sync::{Arc, RwLock};

lazy_static! {
    static ref JAVA_LANG_STRING_VALUE_OFFSET: RwLock<Option<usize>> = { RwLock::new(None) };
}

pub fn set_java_lang_string_value_offset(offset: usize) {
    let mut v = JAVA_LANG_STRING_VALUE_OFFSET.write().unwrap();
    *v = Some(offset);
}

// pub fn is_ref(v: &Oop) -> bool {
//     match v {
//         Oop::Ref(_) => true,
//         _ => false,
//     }
// }

pub fn is_null(v: &Oop) -> bool {
    match v {
        Oop::Null => true,
        _ => false,
    }
}

fn is_str(v: &OopRef) -> bool {
    let v = v.read().unwrap();
    match &v.v {
        oop::RefKind::Inst(inst) => {
            let cls = inst.class.read().unwrap();
            cls.name.as_slice() == b"java/lang/String"
        }
        _ => false,
    }
}

pub fn extract_java_lang_string_value(v: &Oop) -> Vec<u16> {
    let offset = {
        let v = JAVA_LANG_STRING_VALUE_OFFSET.read().unwrap();
        v.clone().unwrap()
    };

    let cls_string = require_class3(None, b"java/lang/String").unwrap();
    let v = {
        let cls = cls_string.read().unwrap();
        cls.get_field_value2(v, offset)
    };

    let v = extract_ref(&v);
    let v = v.read().unwrap();
    match &v.v {
        oop::RefKind::TypeArray(ary) => match ary {
            oop::TypeArrayValue::Char(ary) => ary.to_vec(),
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}

pub fn extract_str(v: &Oop) -> String {
    let value = extract_java_lang_string_value(v);
    String::from_utf16_lossy(value.as_slice())
}

pub fn extract_int(v: &Oop) -> i32 {
    match v {
        Oop::Int(v) => *v,
        _ => unreachable!(),
    }
}

pub fn extract_float(v: &Oop) -> f32 {
    match v {
        Oop::Float(v) => *v,
        _ => unreachable!(),
    }
}

pub fn extract_long(v: &Oop) -> i64 {
    match v {
        Oop::Long(v) => *v,
        _ => unreachable!(),
    }
}

pub fn extract_double(v: &Oop) -> f64 {
    match v {
        Oop::Double(v) => *v,
        _ => unreachable!(),
    }
}

pub fn extract_ref(v: &Oop) -> OopRef {
    match v {
        Oop::Ref(v) => v.clone(),
        t => unreachable!("t = {:?}", t),
    }
}

pub fn if_acmpeq(v1: &Oop, v2: &Oop) -> bool {
    let v1_is_null = is_null(v1);
    let v2_is_null = is_null(v2);

    match (v1_is_null, v2_is_null) {
        (true, true) => return true,
        (true, false) => return false,
        (false, true) => return false,
        (false, false) => (),
    }

    let v1_ref = extract_ref(v1);
    let v2_ref = extract_ref(v2);

    {
        // let v = v1_ref.read().unwrap();
        // error!("xx {:?}", v.v);
    }

    if Arc::ptr_eq(&v1_ref, &v2_ref) {
        true
    } else {
        if is_str(&v1_ref) && is_str(&v2_ref) {
            let v1 = extract_str(v1);
            let v2 = extract_str(v2);
            if v1 == v2 {
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}

pub fn new_java_lang_string2(jt: &mut JavaThread, v: &str) -> Oop {
    //build "char value[]"
    let chars: Vec<u16> = v.as_bytes().iter().map(|v| *v as u16).collect();
    let ary = Oop::char_ary_from1(chars.as_slice());

    //new String(char value[])
    let string_cls = require_class3(None, b"java/lang/String").unwrap();
    let string_oop = Oop::new_inst(string_cls.clone());
    let args = vec![string_oop.clone(), ary];
    runtime::java_call::invoke_ctor(jt, string_cls, b"([C)V", args);

    string_oop
}

pub fn new_java_lang_string3(jt: &mut JavaThread, bs: &[u8]) -> Oop {
    let length = bs.len();
    let mut buffer: Vec<u16> = Vec::with_capacity(length);
    let mut pos = 0;
    while pos < length {
        if bs[pos] & 0x80 == 0 {
            let v = bs[pos] as u16;
            buffer.push(v);
            pos += 1;
        } else if bs[pos] & 0xE0 == 0xC0 && (bs[pos + 1] & 0xC0) == 0x80 {
            let x = bs[pos] as u16;
            let y = bs[pos + 1] as u16;
            let v = ((x & 0x1f) << 6) + (y & 0x3f);
            buffer.push(v);
            pos += 2;
        } else if bs[pos] & 0xF0 == 0xE0
            && (bs[pos + 1] & 0xC0) == 0x80
            && (bs[pos + 2] & 0xC0) == 0x80
        {
            let x = bs[pos] as u16;
            let y = bs[pos + 1] as u16;
            let z = bs[pos + 2] as u16;
            let v = ((x & 0xf) << 12) + ((y & 0x3f) << 6) + (z & 0x3f);
            buffer.push(v);
            pos += 3;
        } else if bs[pos] == 0xED
            && (bs[pos + 1] & 0xF0 == 0xA0)
            && (bs[pos + 2] & 0xC0 == 0x80)
            && (bs[pos + 3] == 0xED)
            && (bs[pos + 4] & 0xF0 == 0xB0)
            && (bs[pos + 5] & 0xC0 == 0x80)
        {
            let v = bs[pos + 1] as u32;
            let w = bs[pos + 2] as u32;
            let y = bs[pos + 4] as u32;
            let z = bs[pos + 5] as u32;
            let vv =
                0x10000 + ((v & 0x0f) << 16) + ((w & 0x3f) << 10) + ((y & 0x0f) << 6) + (z & 0x3f);
            buffer.push(vv as u16);

            pos += 6;
        } else {
            unreachable!()
        }
    }

    //build "char value[]"
    let ary = Oop::char_ary_from1(buffer.as_slice());

    //new String(char value[])
    let string_cls = require_class3(None, b"java/lang/String").unwrap();
    let string_oop = Oop::new_inst(string_cls.clone());
    let args = vec![string_oop.clone(), ary];
    runtime::java_call::invoke_ctor(jt, string_cls, b"([C)V", args);

    string_oop
}

pub fn hash_code(v: &Oop) -> i32 {
    match v {
        Oop::Ref(rf) => {
            if is_str(rf) {
                let value = extract_java_lang_string_value(v);
                return if value.len() == 0 {
                    0
                } else {
                    let mut h = 0i32;
                    for v in value {
                        h = h.wrapping_mul(31).wrapping_add(v as i32);
                    }
                    h
                };
            } else {
                let v = Arc::into_raw(rf.clone());
                v as i32
            }
        }
        Oop::Null => 0,
        _ => unreachable!(),
    }
}
