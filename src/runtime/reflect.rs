#![allow(non_snake_case)]

use crate::classfile::consts as cls_const;
use crate::classfile::signature::{FieldSignature, MethodSignature, Type as ArgType, Type};
use crate::native::java_lang_Class;
use crate::oop::{self, Oop, OopDesc, ValueType};
use crate::runtime::{self, require_class3, JavaThread};
use crate::types::*;
use crate::util;

pub fn new_field(jt: &mut JavaThread, fir: FieldIdRef) -> OopRef {
    let field_cls = runtime::require_class3(None, cls_const::J_FIELD).unwrap();

    let clazz = { fir.field.class.lock().unwrap().get_mirror() };

    let field_sig = FieldSignature::new(fir.field.desc.as_slice());
    let typ_mirror = create_value_type(field_sig.field_type);
    let desc = unsafe { std::str::from_utf8_unchecked(fir.field.desc.as_slice()) };
    let signature = util::oop::new_java_lang_string2(jt, desc);

    let field_name = unsafe { std::str::from_utf8_unchecked(fir.field.name.as_slice()) };
    let mut desc = Vec::new();
    desc.push(b'(');
    let mut args: Vec<OopRef> = vec![
        ("clazz", "Ljava/lang/Class;", clazz),
        (
            "name",
            "Ljava/lang/String;",
            util::oop::new_java_lang_string2(jt, field_name),
        ),
        ("type", "Ljava/lang/Class;", typ_mirror),
        (
            "modifiers",
            "I",
            OopDesc::new_int(fir.field.acc_flags as i32),
        ),
        ("slot", "I", OopDesc::new_int(fir.offset as i32)),
        ("signature", "Ljava/lang/String;", signature),
        ("annotations", "[B", oop::consts::get_null()),
    ]
    .iter()
    .map(|(_, t, v)| {
        desc.extend_from_slice(t.as_bytes());
        v.clone()
    })
    .collect();
    desc.extend_from_slice(")V".as_bytes());

    let oop = OopDesc::new_inst(field_cls.clone());
    args.insert(0, oop.clone());
    runtime::java_call::invoke_ctor(jt, field_cls, desc.as_slice(), args);

    oop
}

pub fn new_method_ctor(jt: &mut JavaThread, mir: MethodIdRef) -> OopRef {
    let ctor_cls = require_class3(None, cls_const::J_METHOD_CTOR).unwrap();

    //declaringClass
    let declaring_cls = { mir.method.class.lock().unwrap().get_mirror() };

    //parameterTypes
    let signature = MethodSignature::new(mir.method.desc.as_slice());
    let params: Vec<OopRef> = signature
        .args
        .iter()
        .map(|t| create_value_type(t.clone()))
        .collect();
    let cls = require_class3(None, b"[Ljava/lang/Class;").unwrap();
    let parameter_types = OopDesc::new_ref_ary2(cls, params);

    //fixme: checkedExceptions
    let cls = require_class3(None, b"[Ljava/lang/Class;").unwrap();
    let checked_exceptions = OopDesc::new_ref_ary2(cls, vec![]);

    //modifiers
    let modifiers = mir.method.acc_flags;
    //slot
    let slot = mir.offset;
    //signature
    let desc = unsafe { std::str::from_utf8_unchecked(mir.method.desc.as_slice()) };
    let signature = util::oop::new_java_lang_string2(jt, desc);
    //fixme:
    let annotations = OopDesc::new_byte_ary(0);
    let parameter_annotations = OopDesc::new_byte_ary(0);

    let mut desc = Vec::new();
    desc.push(b'(');
    let mut args: Vec<OopRef> = vec![
        ("declaringClass", "Ljava/lang/Class;", declaring_cls),
        ("parameterTypes", "[Ljava/lang/Class;", parameter_types),
        (
            "checkedExceptions",
            "[Ljava/lang/Class;",
            checked_exceptions,
        ),
        ("modifiers", "I", OopDesc::new_int(modifiers as i32)),
        ("slot", "I", OopDesc::new_int(slot as i32)),
        ("signature", "Ljava/lang/String;", signature),
        ("annotations", "[B", annotations),
        ("parameterAnnotations", "[B", parameter_annotations),
    ]
    .iter()
    .map(|(_, t, v)| {
        desc.extend_from_slice(t.as_bytes());
        v.clone()
    })
    .collect();
    desc.extend_from_slice(")V".as_bytes());

    let oop = OopDesc::new_inst(ctor_cls.clone());
    args.insert(0, oop.clone());
    runtime::java_call::invoke_ctor(jt, ctor_cls, desc.as_slice(), args);

    oop
}

pub fn get_Constructor_clazz(ctor: OopRef) -> OopRef {
    //todo: optimize, avoid obtain class
    let cls = {
        let v = ctor.lock().unwrap();
        match &v.v {
            Oop::Inst(inst) => inst.class.clone(),
            _ => unreachable!(),
        }
    };

    //todo: optimize, avoid obtain id
    let cls = cls.lock().unwrap();
    let id = cls.get_field_id(b"clazz", b"Ljava/lang/Class;", false);
    cls.get_field_value(ctor, id)
}

pub fn get_Constructor_slot(ctor: OopRef) -> i32 {
    let cls = {
        let v = ctor.lock().unwrap();
        match &v.v {
            Oop::Inst(inst) => inst.class.clone(),
            _ => unreachable!(),
        }
    };

    let cls = cls.lock().unwrap();
    let id = cls.get_field_id(b"slot", b"I", false);
    let v = cls.get_field_value(ctor, id);
    let v = v.lock().unwrap();
    match v.v {
        Oop::Int(v) => v,
        _ => unreachable!(),
    }
}

pub fn get_Constructor_signature(ctor: OopRef) -> String {
    //todo: optimisze, cache Constructor cls, avoid obtain class
    let cls = {
        let v = ctor.lock().unwrap();
        match &v.v {
            Oop::Inst(inst) => inst.class.clone(),
            _ => unreachable!(),
        }
    };

    //todo: optimize, cache id
    let cls = cls.lock().unwrap();
    let id = cls.get_field_id(b"signature", b"Ljava/lang/String;", false);
    let v = cls.get_field_value(ctor, id);
    util::oop::extract_str(v)
}

fn create_value_type(t: ArgType) -> OopRef {
    match t {
        Type::Byte => java_lang_Class::get_primitive_class_mirror("B").unwrap(),
        Type::Char => java_lang_Class::get_primitive_class_mirror("C").unwrap(),
        Type::Int => java_lang_Class::get_primitive_class_mirror("I").unwrap(),
        Type::Double => java_lang_Class::get_primitive_class_mirror("D").unwrap(),
        Type::Float => java_lang_Class::get_primitive_class_mirror("F").unwrap(),
        Type::Long => java_lang_Class::get_primitive_class_mirror("J").unwrap(),
        Type::Object(desc) => {
            let len = desc.len();
            let name = &desc.as_slice()[1..len - 1];
            let cls = require_class3(None, name).unwrap();
            let cls = cls.lock().unwrap();
            cls.get_mirror()
        }
        Type::Short => java_lang_Class::get_primitive_class_mirror("S").unwrap(),
        Type::Boolean => java_lang_Class::get_primitive_class_mirror("Z").unwrap(),
        Type::Array(desc) => {
            let cls = require_class3(None, desc.as_slice()).unwrap();
            let cls = cls.lock().unwrap();
            cls.get_mirror()
        }
        Type::Void => java_lang_Class::get_primitive_class_mirror("V").unwrap(),
    }
}
