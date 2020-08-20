use crate::new_br;
use crate::oop::Oop;
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

pub fn new_java_lang_string2(v: &str) -> Oop {
    //build "char value[]"
    let chars: Vec<u16> = v.as_bytes().iter().map(|v| *v as u16).collect();
    let ary = Oop::char_ary_from1(chars.as_slice());

    //new String(char value[])
    let string_cls = require_class3(None, b"java/lang/String").unwrap();
    let string_oop = Oop::new_inst(string_cls.clone());
    let args = vec![string_oop.clone(), ary];
    runtime::invoke::invoke_ctor(string_cls, util::S_NEW_STRING_SIG.clone(), args);

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
    runtime::invoke::invoke_ctor(string_cls, util::S_NEW_STRING_SIG.clone(), args);

    string_oop
}
