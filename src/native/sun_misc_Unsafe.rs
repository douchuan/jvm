#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{self, Oop, OopDesc, OopRef};
use crate::runtime::{require_class3, JavaThread};
use crate::util;
use std::sync::{Arc, Mutex};

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![
        new_fn("registerNatives", "()V", Box::new(jvm_registerNatives)),
        new_fn(
            "arrayBaseOffset",
            "(Ljava/lang/Class;)I",
            Box::new(jvm_arrayBaseOffset),
        ),
        new_fn(
            "arrayIndexScale",
            "(Ljava/lang/Class;)I",
            Box::new(jvm_arrayIndexScale),
        ),
        new_fn("addressSize", "()I", Box::new(jvm_addressSize)),
        new_fn(
            "objectFieldOffset",
            "(Ljava/lang/reflect/Field;)J",
            Box::new(jvm_objectFieldOffset),
        ),
        new_fn(
            "compareAndSwapObject",
            "(Ljava/lang/Object;JLjava/lang/Object;Ljava/lang/Object;)Z",
            Box::new(jvm_compareAndSwapObject),
        ),
    ]
}

fn jvm_registerNatives(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    Ok(None)
}

fn jvm_arrayBaseOffset(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    Ok(Some(OopDesc::new_int(0)))
}

fn jvm_arrayIndexScale(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    //fixme:
    Ok(Some(OopDesc::new_int(4)))
}

fn jvm_addressSize(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    //fixme:
    Ok(Some(OopDesc::new_int(4)))
}

fn jvm_objectFieldOffset(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let field = args[1].clone();

    {
        let v = field.lock().unwrap();
        match &v.v {
            Oop::Inst(inst) => {
                let cls = inst.class.clone();
                let cls = cls.lock().unwrap();
                assert_eq!(cls.name.as_slice(), b"java/lang/reflect/Field");
            }
            _ => unreachable!(),
        }
    }

    let cls = require_class3(None, b"java/lang/reflect/Field").unwrap();
    let v = {
        let cls = cls.lock().unwrap();
        let id = cls.get_field_id(b"slot", b"I", false);
        cls.get_field_value(field, id)
    };
    let v = v.lock().unwrap();
    let v = match &v.v {
        Oop::Int(i) => OopDesc::new_long(*i as i64),
        _ => unreachable!(),
    };

    Ok(Some(v))
}

//fixme: 此处语义上要求是原子操作，这里需要重新实现
fn jvm_compareAndSwapObject(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let owner = args.get(1).unwrap();
    let offset = {
        let v = args.get(2).unwrap();
        let v = v.lock().unwrap();
        match v.v {
            Oop::Long(v) => v,
            _ => unreachable!(),
        }
    };
    let old_data = args.get(3).unwrap();
    let new_data = args.get(4).unwrap();

    let v_at_offset = {
        let v = owner.lock().unwrap();
        match &v.v {
            Oop::Mirror(mirror) => mirror.field_values[offset as usize].clone(),
            _ => unreachable!(),
        }
    };

    if util::oop::if_acmpeq(v_at_offset, old_data.clone()) {
        let mut v = owner.lock().unwrap();
        match &mut v.v {
            Oop::Mirror(mirror) => {
                mirror.field_values[offset as usize] = new_data.clone();
            }
            _ => unreachable!(),
        }

        Ok(Some(OopDesc::new_int(1)))
    } else {
        Ok(Some(OopDesc::new_int(0)))
    }
}
