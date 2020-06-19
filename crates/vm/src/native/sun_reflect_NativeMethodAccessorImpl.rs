#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::new_br;
use crate::oop::{self, Class, Oop, OopRef};
use crate::runtime::{self, require_class3};
use classfile::{consts as cls_consts, SignatureType};

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![new_fn(
        "invoke0",
        "(Ljava/lang/reflect/Method;Ljava/lang/Object;[Ljava/lang/Object;)Ljava/lang/Object;",
        Box::new(jvm_invoke0),
    )]
}

fn jvm_invoke0(_env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    let method = args.get(0).unwrap();
    let obj = args.get(1).unwrap();
    let args = args.get(2).unwrap();

    let cls = require_class3(None, cls_consts::J_METHOD).unwrap();

    let (m_clazz, m_name, m_signature) = {
        let cls = cls.get_class();

        let fid = cls.get_field_id(new_br("clazz"), new_br("Ljava/lang/Class;"), false);
        let method_clazz = Class::get_field_value(method.extract_ref(), fid);

        let fid = cls.get_field_id(new_br("name"), new_br("Ljava/lang/String;"), false);
        let method_name = Class::get_field_value(method.extract_ref(), fid);
        let method_name = OopRef::java_lang_string(method_name.extract_ref());
        let method_name = new_br(method_name.as_str());

        let fid = cls.get_field_id(new_br("signature"), new_br("Ljava/lang/String;"), false);
        let signature = Class::get_field_value(method.extract_ref(), fid);
        let signature = OopRef::java_lang_string(signature.extract_ref());
        let signature = new_br(signature.as_str());

        (method_clazz, method_name, signature)
    };

    let clz = {
        let rf = m_clazz.extract_ref();
        let mirror = rf.extract_mirror();
        mirror.target.clone().unwrap()
    };

    let mir = {
        let clz = clz.get_class();
        clz.get_class_method(m_name, m_signature).unwrap()
    };

    // {
    //     let cls = clz.read().unwrap();
    //     let cls_name = cls.name.clone();
    //     error!(
    //         "invoke0 {}:{}:{} static={}, native={}",
    //         String::from_utf8_lossy(cls_name.as_slice()),
    //         String::from_utf8_lossy(mir.method.name.as_slice()),
    //         String::from_utf8_lossy(mir.method.desc.as_slice()),
    //         mir.method.is_static(),
    //         mir.method.is_native(),
    //     );
    // }

    let mut args = {
        let rf = args.extract_ref();
        let ary = rf.extract_array();
        ary.elements.to_vec()
    };

    if !mir.method.is_static() {
        args.insert(0, obj.clone());
    }

    let force_no_resolve = mir.method.name.as_slice() == b"<init>" || mir.method.is_static();
    let mut jc = runtime::invoke::JavaCall::new_with_args(mir, args);
    let area = runtime::DataArea::new(0, 0);
    jc.invoke(Some(area.clone()), force_no_resolve);

    // error!("invoke0 return_type = {:?}, desc={}", jc.return_type, String::from_utf8_lossy(mir.method.desc.as_slice()));

    let r = {
        let mut area = area.write().unwrap();
        match jc.return_type {
            SignatureType::Byte
            | SignatureType::Char
            | SignatureType::Boolean
            | SignatureType::Short
            | SignatureType::Int => {
                let v = area.stack.pop_int();
                Some(oop::Oop::new_int(v))
            }
            SignatureType::Double => {
                let v = area.stack.pop_double();
                Some(oop::Oop::new_double(v))
            }
            SignatureType::Float => {
                let v = area.stack.pop_float();
                Some(oop::Oop::new_float(v))
            }
            SignatureType::Long => {
                let v = area.stack.pop_long();
                Some(oop::Oop::new_long(v))
            }
            SignatureType::Object(_, _, _) | SignatureType::Array(_) => Some(area.stack.pop_ref()),
            SignatureType::Void => Some(oop::consts::get_null()),
        }
    };

    Ok(r)
}
