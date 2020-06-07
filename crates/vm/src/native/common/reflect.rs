#![allow(non_snake_case)]

use crate::native::java_lang_Class;
use crate::oop::{self, Class, Oop, OopRef};
use crate::runtime::{self, require_class3};
use crate::types::*;
use crate::{util, new_br};
use class_parser::{FieldSignature, MethodSignature};
use classfile::consts as cls_const;
use classfile::SignatureType;
use std::sync::Arc;

pub fn new_field(fir: FieldIdRef) -> Oop {
    let field_cls = runtime::require_class3(None, cls_const::J_FIELD).unwrap();

    let clazz = fir.field.class.read().unwrap().get_mirror();

    let field_sig = FieldSignature::new(fir.field.desc.as_slice());
    let typ_mirror = create_value_type(field_sig.field_type);
    let desc = unsafe { std::str::from_utf8_unchecked(fir.field.desc.as_slice()) };
    let signature = util::oop::new_java_lang_string2(desc);

    let field_name = unsafe { std::str::from_utf8_unchecked(fir.field.name.as_slice()) };
    let mut desc = Vec::new();
    desc.push(b'(');
    let mut args: Vec<Oop> = vec![
        ("clazz", "Ljava/lang/Class;", clazz),
        (
            "name",
            "Ljava/lang/String;",
            util::oop::new_java_lang_string2(field_name),
        ),
        ("type", "Ljava/lang/Class;", typ_mirror),
        ("modifiers", "I", Oop::new_int(fir.field.acc_flags as i32)),
        ("slot", "I", Oop::new_int(fir.offset as i32)),
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

    let oop = Oop::new_inst(field_cls.clone());
    args.insert(0, oop.clone());
    runtime::invoke::invoke_ctor(field_cls, Arc::new(desc), args);

    oop
}

pub fn new_method_ctor(mir: MethodIdRef) -> Oop {
    let ctor_cls = require_class3(None, cls_const::J_METHOD_CTOR).unwrap();

    //declaringClass
    let declaring_cls = mir.method.class.read().unwrap().get_mirror();

    //parameterTypes
    let signature = MethodSignature::new(mir.method.desc.as_slice());
    let params: Vec<Oop> = signature
        .args
        .iter()
        .map(|t| create_value_type(t.clone()))
        .collect();
    let cls = require_class3(None, b"[Ljava/lang/Class;").unwrap();
    let parameter_types = Oop::new_ref_ary2(cls, params);

    //fixme: checkedExceptions
    let cls = require_class3(None, b"[Ljava/lang/Class;").unwrap();
    let checked_exceptions = Oop::new_ref_ary2(cls, vec![]);

    //modifiers
    let modifiers = mir.method.acc_flags;
    //slot
    let slot = mir.offset;
    //signature
    let desc = unsafe { std::str::from_utf8_unchecked(mir.method.desc.as_slice()) };
    let signature = util::oop::new_java_lang_string2(desc);
    let annotations = {
        let raw = mir.method.get_annotation();
        match raw {
            Some(raw) => Oop::new_byte_ary2(raw.to_vec()),
            None => oop::consts::get_null(),
        }
    };
    let parameter_annotations = {
        let raw = mir.method.get_param_annotation();
        match raw {
            Some(raw) => Oop::new_byte_ary2(raw.to_vec()),
            None => oop::consts::get_null(),
        }
    };

    let mut desc = Vec::new();
    desc.push(b'(');
    let mut args: Vec<Oop> = vec![
        ("declaringClass", "Ljava/lang/Class;", declaring_cls),
        ("parameterTypes", "[Ljava/lang/Class;", parameter_types),
        (
            "checkedExceptions",
            "[Ljava/lang/Class;",
            checked_exceptions,
        ),
        ("modifiers", "I", Oop::new_int(modifiers as i32)),
        ("slot", "I", Oop::new_int(slot as i32)),
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

    let oop = Oop::new_inst(ctor_cls.clone());
    args.insert(0, oop.clone());
    runtime::invoke::invoke_ctor(ctor_cls, Arc::new(desc), args);

    oop
}

pub fn new_method_normal(mir: MethodIdRef) -> Oop {
    let ctor_cls = require_class3(None, cls_const::J_METHOD).unwrap();

    //declaringClass
    let declaring_cls = mir.method.class.read().unwrap().get_mirror();

    //name
    let name = {
        let name = unsafe { std::str::from_utf8_unchecked(mir.method.name.as_slice()) };
        util::oop::new_java_lang_string2(name)
    };

    //parameterTypes
    let signature = MethodSignature::new(mir.method.desc.as_slice());
    let params: Vec<Oop> = signature
        .args
        .iter()
        .map(|t| create_value_type(t.clone()))
        .collect();
    let cls = require_class3(None, b"[Ljava/lang/Class;").unwrap();
    let parameter_types = Oop::new_ref_ary2(cls, params);

    //returnType
    let return_type = create_value_type(signature.retype.clone());

    //fixme: checkedExceptions
    let cls = require_class3(None, b"[Ljava/lang/Class;").unwrap();
    let checked_exceptions = Oop::new_ref_ary2(cls, vec![]);

    //modifiers
    let modifiers = mir.method.acc_flags;
    //slot
    let slot = mir.offset;
    //signature
    let signature = {
        let desc = unsafe { std::str::from_utf8_unchecked(mir.method.desc.as_slice()) };
        util::oop::new_java_lang_string2(desc)
    };
    let annotations = {
        let raw = mir.method.get_annotation();
        match raw {
            Some(raw) => Oop::new_byte_ary2(raw.to_vec()),
            None => oop::consts::get_null(),
        }
    };
    let parameter_annotations = {
        let raw = mir.method.get_param_annotation();
        match raw {
            Some(raw) => Oop::new_byte_ary2(raw.to_vec()),
            None => oop::consts::get_null(),
        }
    };
    let annotation_default = {
        let raw = mir.method.get_annotation_default();
        match raw {
            Some(raw) => Oop::new_byte_ary2(raw.to_vec()),
            None => oop::consts::get_null(),
        }
    };

    let mut desc = Vec::new();
    desc.push(b'(');
    let mut args: Vec<Oop> = vec![
        ("declaringClass", "Ljava/lang/Class;", declaring_cls),
        ("name", "Ljava/lang/String;", name),
        ("parameterTypes", "[Ljava/lang/Class;", parameter_types),
        ("returnType", "Ljava/lang/Class;", return_type),
        (
            "checkedExceptions",
            "[Ljava/lang/Class;",
            checked_exceptions,
        ),
        ("modifiers", "I", Oop::new_int(modifiers as i32)),
        ("slot", "I", Oop::new_int(slot as i32)),
        ("signature", "Ljava/lang/String;", signature),
        ("annotations", "[B", annotations),
        ("parameterAnnotations", "[B", parameter_annotations),
        ("annotationDefault", "[B", annotation_default),
    ]
    .iter()
    .map(|(_, t, v)| {
        desc.extend_from_slice(t.as_bytes());
        v.clone()
    })
    .collect();
    desc.extend_from_slice(")V".as_bytes());

    let oop = Oop::new_inst(ctor_cls.clone());
    args.insert(0, oop.clone());
    runtime::invoke::invoke_ctor(ctor_cls, Arc::new(desc), args);

    oop
}

pub fn get_Constructor_clazz(ctor: &Oop) -> Oop {
    //todo: optimize, avoid obtain class
    let cls = {
        let rf = ctor.extract_ref();
        let inst = rf.extract_inst();
        inst.class.clone()
    };

    //todo: optimize, avoid obtain id
    let cls = cls.read().unwrap();
    let id = cls.get_field_id(new_br("clazz"), new_br("Ljava/lang/Class;"), false);
    Class::get_field_value(ctor.extract_ref(), id)
}

/*
pub fn get_Constructor_slot(ctor: &Oop) -> i32 {
    let cls = {
        let v = util::oop::extract_ref(ctor);
        let v = v.read().unwrap();
        match &v.v {
            oop::RefKind::Inst(inst) => inst.class.clone(),
            _ => unreachable!(),
        }
    };

    let cls = cls.read().unwrap();
    let id = cls.get_field_id(b"slot", b"I", false);
    let v = cls.get_field_value(ctor, id);
    util::oop::extract_int(&v)
}
*/

pub fn get_Constructor_signature(ctor: &Oop) -> String {
    //todo: optimisze, cache Constructor cls, avoid obtain class
    let cls = {
        let rf = ctor.extract_ref();
        let inst = rf.extract_inst();
        inst.class.clone()
    };

    //todo: optimize, cache id
    let cls = cls.read().unwrap();
    let id = cls.get_field_id(new_br("signature"), new_br("Ljava/lang/String;"), false);
    let v = Class::get_field_value(ctor.extract_ref(), id);
    OopRef::java_lang_string(v.extract_ref())
}

fn create_value_type(t: SignatureType) -> Oop {
    match t {
        SignatureType::Byte => java_lang_Class::get_primitive_class_mirror("B").unwrap(),
        SignatureType::Char => java_lang_Class::get_primitive_class_mirror("C").unwrap(),
        SignatureType::Int => java_lang_Class::get_primitive_class_mirror("I").unwrap(),
        SignatureType::Double => java_lang_Class::get_primitive_class_mirror("D").unwrap(),
        SignatureType::Float => java_lang_Class::get_primitive_class_mirror("F").unwrap(),
        SignatureType::Long => java_lang_Class::get_primitive_class_mirror("J").unwrap(),
        SignatureType::Object(desc, _, _) => {
            let len = desc.len();
            let name = &desc.as_slice()[1..len - 1];
            let cls = require_class3(None, name).unwrap();
            let cls = cls.read().unwrap();
            cls.get_mirror()
        }
        SignatureType::Short => java_lang_Class::get_primitive_class_mirror("S").unwrap(),
        SignatureType::Boolean => java_lang_Class::get_primitive_class_mirror("Z").unwrap(),
        SignatureType::Array(desc) => {
            let cls = require_class3(None, desc.as_slice()).unwrap();
            let cls = cls.read().unwrap();
            cls.get_mirror()
        }
        SignatureType::Void => java_lang_Class::get_primitive_class_mirror("V").unwrap(),
    }
}
