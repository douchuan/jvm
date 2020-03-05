#![allow(non_snake_case)]

use crate::classfile::signature::Type;
use crate::classfile::consts as cls_consts;
use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{self, Oop};
use crate::runtime::{self, require_class3, JavaThread};
use crate::util;

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![new_fn(
        "invoke0",
        "(Ljava/lang/reflect/Method;Ljava/lang/Object;[Ljava/lang/Object;)Ljava/lang/Object;",
        Box::new(jvm_invoke0),
    )]
}

fn jvm_invoke0(jt: &mut JavaThread, _env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    let method = args.get(0).unwrap();
    let obj = args.get(1).unwrap();
    let args = args.get(2).unwrap();

    let cls = require_class3(None, cls_consts::J_METHOD).unwrap();

    let (m_clazz, m_name, m_signature) = {
        let cls = cls.read().unwrap();

        let fid = cls.get_field_id(b"clazz", b"Ljava/lang/Class;", false);
        let method_clazz = cls.get_field_value(&method, fid);

        let fid = cls.get_field_id(b"name", b"Ljava/lang/String;", false);
        let method_name = cls.get_field_value(&method, fid);
        let method_name = util::oop::extract_str(&method_name);

        let fid = cls.get_field_id(b"signature", b"Ljava/lang/String;", false);
        let signature = cls.get_field_value(&method, fid);
        let signature = util::oop::extract_str(&signature);

        (method_clazz, method_name, signature)
    };

    let clz = {
        match m_clazz {
            oop::Oop::Ref(rf) => {
                let rf = rf.read().unwrap();
                match &rf.v {
                    oop::RefKind::Mirror(mirror) => mirror.target.clone().unwrap(),
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    };

    let mir = {
        let clz = clz.read().unwrap();
        let id = util::new_method_id(m_name.as_bytes(), m_signature.as_bytes());
        clz.get_class_method(id).unwrap()
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
        match args {
            oop::Oop::Ref(rf) => {
                let rf = rf.read().unwrap();
                match &rf.v {
                    oop::RefKind::Array(ary) => ary.elements.clone(),
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    };

    if !mir.method.is_static() {
        args.insert(0, obj.clone());
    }
    let force_no_resolve = mir.method.name.as_slice() == b"<init>" || mir.method.is_static();
    let mut jc = runtime::java_call::JavaCall::new_with_args(jt, mir, args);
    let area = runtime::DataArea::new(0, 0);
    jc.invoke(jt, Some(&area), force_no_resolve);

    let r = {
        let mut area = area.borrow_mut();
        match jc.return_type {
            Type::Byte | Type::Char | Type::Boolean | Type::Short | Type::Int => {
                let v = area.stack.pop_int();
                Some(oop::Oop::new_int(v))
            }
            Type::Double => {
                let v = area.stack.pop_double();
                Some(oop::Oop::new_double(v))
            }
            Type::Float => {
                let v = area.stack.pop_float();
                Some(oop::Oop::new_float(v))
            }
            Type::Long => {
                let v = area.stack.pop_long();
                Some(oop::Oop::new_long(v))
            }
            Type::Object(_) | Type::Array(_) => Some(area.stack.pop_ref()),
            Type::Void => None,
        }
    };

    Ok(r)
}
