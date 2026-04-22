use crate::oop::class::Class;
use crate::oop::{self, Oop};
use crate::runtime::{self, require_class3};
use crate::util;

static mut JAVA_LANG_STRING_VALUE_OFFSET: usize = 0;
static mut JAVA_LANG_INTEGER_VALUE_OFFSET: usize = 0;

pub fn set_java_lang_string_value_offset(offset: usize) {
    unsafe {
        JAVA_LANG_STRING_VALUE_OFFSET = offset;
    }
}

pub fn set_java_lang_integer_value_offset(offset: usize) {
    unsafe {
        JAVA_LANG_INTEGER_VALUE_OFFSET = offset;
    }
}

pub fn get_java_lang_string_value_offset() -> usize {
    unsafe { JAVA_LANG_STRING_VALUE_OFFSET }
}

pub fn get_java_lang_integer_value_offset() -> usize {
    unsafe { JAVA_LANG_INTEGER_VALUE_OFFSET }
}

pub fn new_java_lang_string_direct(v: &str) -> Oop {
    use crate::new_br;

    let string_cls = require_class3(None, b"java/lang/String").unwrap();
    let string_oop = Oop::new_inst(string_cls.clone());
    let slot_id = match string_oop {
        Oop::Ref(id) => id,
        _ => unreachable!(),
    };
    let cls = string_cls.get_class();

    // Try JDK 9+ byte[] value field
    if let Ok(value_fid) = cls.get_field_id_safe(&new_br("value"), &new_br("[B"), false) {
        let bytes = v.as_bytes().to_vec();
        let byte_ary = Oop::new_byte_ary2(bytes);
        Class::put_field_value2(slot_id, value_fid.offset, byte_ary);

        let coder = if v.bytes().all(|b| b < 128) {
            0i32
        } else {
            1i32
        };
        if let Ok(coder_fid) = cls.get_field_id_safe(&new_br("coder"), &new_br("B"), false) {
            Class::put_field_value2(slot_id, coder_fid.offset, Oop::Int(coder));
        }
    } else {
        let chars: Vec<u16> = v.encode_utf16().collect();
        let ary = Oop::char_ary_from1(&chars);
        if let Ok(value_fid) = cls.get_field_id_safe(&new_br("value"), &new_br("[C"), false) {
            Class::put_field_value2(slot_id, value_fid.offset, ary);
        }
    }

    string_oop
}

pub fn new_java_lang_string2(v: &str) -> Oop {
    //build "char value[]"
    let chars: Vec<u16> = v.as_bytes().iter().map(|v| *v as u16).collect();
    let ary = Oop::char_ary_from1(chars.as_slice());

    //new String(char value[])
    let string_cls = require_class3(None, b"java/lang/String").unwrap();
    let string_oop = Oop::new_inst(string_cls.clone());
    let args = vec![string_oop.clone(), ary];
    runtime::invoke::invoke_ctor(
        string_cls,
        util::S_NEW_STRING_SIG.get().unwrap().clone(),
        args,
    );

    string_oop
}

pub fn new_java_lang_string3(bs: &[u8]) -> Oop {
    let buffer = classfile::constant_pool::construct_string_raw(bs);

    //build "char value[]"
    let ary = Oop::char_ary_from1(buffer.as_slice());

    //new String(char value[])
    let string_cls = require_class3(None, b"java/lang/String").unwrap();
    let string_oop = Oop::new_inst(string_cls.clone());
    let args = vec![string_oop.clone(), ary];
    runtime::invoke::invoke_ctor(
        string_cls,
        util::S_NEW_STRING_SIG.get().unwrap().clone(),
        args,
    );

    string_oop
}
