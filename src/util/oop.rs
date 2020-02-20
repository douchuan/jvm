use crate::oop::{self, Oop, OopDesc};
use crate::runtime::{self, require_class3, JavaThread};
use crate::types::OopRef;
use crate::util;
use std::sync::{Arc, Mutex};

lazy_static! {
    static ref JAVA_LANG_STRING_VALUE_OFFSET: Mutex<Option<usize>> = { Mutex::new(None) };
}

pub fn set_java_lang_string_value_offset(offset: usize) {
    util::sync_call_ctx(&JAVA_LANG_STRING_VALUE_OFFSET, |v| {
        *v = Some(offset);
    });
}

pub fn is_str(v: OopRef) -> bool {
    let v = v.lock().unwrap();
    match &v.v {
        Oop::Inst(inst) => {
            let cls = inst.class.lock().unwrap();
            cls.name.as_slice() == b"java/lang/String"
        }
        _ => false,
    }
}

pub fn extract_java_lang_string_value(v: OopRef) -> Vec<u16> {
    let offset: Option<usize> = util::sync_call(&JAVA_LANG_STRING_VALUE_OFFSET, |v| v.clone());
    let offset = offset.unwrap();

    let cls_string = require_class3(None, b"java/lang/String").unwrap();
    let value_ary = {
        let cls = cls_string.lock().unwrap();
        cls.get_field_value2(v.clone(), offset)
    };

    let value_ary = value_ary.lock().unwrap();
    match &value_ary.v {
        Oop::TypeArray(ary) => match ary {
            oop::TypeArrayValue::Char(ary) => Vec::from(ary.as_slice()),
            t => unreachable!("t = {:?}", t),
        },
        _ => unreachable!(),
    }
}

pub fn extract_str(v: OopRef) -> String {
    let value = extract_java_lang_string_value(v);
    String::from_utf16_lossy(value.as_slice())

    /*
    if offset.is_some() {
        let offset = offset.unwrap();

        let cls_string = require_class3(None, b"java/lang/String").unwrap();
        let value_ary = {
            let cls = cls_string.lock().unwrap();
            cls.get_field_value2(v.clone(), offset)
        };

        let value_ary = value_ary.lock().unwrap();
        match &value_ary.v {
            Oop::TypeArray(ary) => match ary {
                oop::TypeArrayValue::Char(ary) => String::from_utf16_lossy(ary.as_slice()),
                t => unreachable!("t = {:?}", t),
            },
            _ => unreachable!(),
        }
    } else {
        let fid = {
            let v = v.lock().unwrap();
            match &v.v {
                Oop::Inst(inst) => {
                    let cls = inst.class.lock().unwrap();
                    cls.get_field_id(b"value", b"[C", false)
                }
                t => unreachable!("t = {:?}", t),
            }
        };

        let cls_string = require_class3(None, b"java/lang/String").unwrap();
        let value_ary = {
            let cls = cls_string.lock().unwrap();
            cls.get_field_value(v.clone(), fid)
        };

        let value_ary = value_ary.lock().unwrap();
        match &value_ary.v {
            Oop::TypeArray(ary) => match ary {
                oop::TypeArrayValue::Char(ary) => String::from_utf16_lossy(ary.as_slice()),
                t => unreachable!("t = {:?}", t),
            },
            _ => unreachable!(),
        }
    }
    */
}

pub fn if_acmpeq(v1: OopRef, v2: OopRef) -> bool {
    if Arc::ptr_eq(&v1, &v2) {
        true
    } else {
        if is_str(v2.clone()) && is_str(v1.clone()) {
            let v2 = extract_str(v2.clone());
            let v1 = extract_str(v1.clone());
            if v2 == v1 {
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}

pub fn new_java_lang_string2(jt: &mut JavaThread, v: &str) -> OopRef {
    //build "char value[]"
    let chars: Vec<u16> = v.as_bytes().iter().map(|v| *v as u16).collect();
    let ary = OopDesc::char_ary_from1(chars.as_slice());

    //new String(char value[])
    let string_cls = require_class3(None, b"java/lang/String").unwrap();
    let string_oop = OopDesc::new_inst(string_cls.clone());
    let args = vec![string_oop.clone(), ary];
    runtime::java_call::invoke_ctor(jt, string_cls, b"([C)V", args);

    string_oop
}

pub fn new_java_lang_string3(jt: &mut JavaThread, bs: &[u8]) -> OopRef {
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
    let ary = OopDesc::char_ary_from1(buffer.as_slice());

    //new String(char value[])
    let string_cls = require_class3(None, b"java/lang/String").unwrap();
    let string_oop = OopDesc::new_inst(string_cls.clone());
    let args = vec![string_oop.clone(), ary];
    runtime::java_call::invoke_ctor(jt, string_cls, b"([C)V", args);

    string_oop
}

pub fn hash_code(v: OopRef) -> i32 {
    {
        let v = v.lock().unwrap();
        match v.v {
            Oop::Null => return 0,
            Oop::Int(_) | Oop::Long(_) | Oop::Float(_) | Oop::Double(_) => unreachable!(),
            _ => (),
        }
    }

    if is_str(v.clone()) {
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
        let v = Arc::into_raw(v);
        v as i32
    }
}
