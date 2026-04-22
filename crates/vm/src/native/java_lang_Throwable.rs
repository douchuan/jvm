#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::new_br;
use crate::oop::{self, Class, Oop};

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![
        new_fn(
            "fillInStackTrace",
            "()Ljava/lang/Throwable;",
            Box::new(jvm_fillInStackTrace),
        ),
        // JDK 9+ internal variant with dummy int parameter
        new_fn(
            "fillInStackTrace",
            "(I)Ljava/lang/Throwable;",
            Box::new(jvm_fillInStackTrace),
        ),
        new_fn(
            "getStackTraceDepth",
            "()I",
            Box::new(jvm_getStackTraceDepth),
        ),
        new_fn(
            "getStackTraceElement",
            "(I)Ljava/lang/StackTraceElement;",
            Box::new(jvm_getStackTraceElement),
        ),
    ]
}

fn jvm_fillInStackTrace(_env: JNIEnv, args: &[Oop]) -> JNIResult {
    Ok(Some(args.get(0).unwrap().clone()))
}

fn jvm_getStackTraceDepth(_env: JNIEnv, args: &[Oop]) -> JNIResult {
    let throwable = args.get(0).unwrap();
    let cls = {
        let rf = throwable.extract_ref();
        oop::with_heap(|heap| {
            let desc = heap.get(rf);
            let guard = desc.read().unwrap();
            guard.v.extract_inst().class.clone()
        })
    };
    let backtrace = {
        let cls = cls.get_class();
        let id = cls.get_field_id(&new_br("backtrace"), &new_br("Ljava/lang/Object;"), false);
        Class::get_field_value(throwable.extract_ref(), id)
    };

    let v = match backtrace {
        Oop::Null => Oop::new_int(0),
        Oop::Ref(rf) => {
            let len = oop::with_heap(|heap| {
                let desc = heap.get(rf);
                let guard = desc.read().unwrap();
                guard.v.extract_array().elements.len()
            });
            Oop::new_int(len as i32)
        }
        _ => unreachable!(),
    };

    Ok(Some(v))
}

fn jvm_getStackTraceElement(_env: JNIEnv, args: &[Oop]) -> JNIResult {
    let throwable = args.get(0).unwrap();
    let index = args.get(1).unwrap().extract_int();
    let cls = {
        let rf = throwable.extract_ref();
        oop::with_heap(|heap| {
            let desc = heap.get(rf);
            let guard = desc.read().unwrap();
            guard.v.extract_inst().class.clone()
        })
    };
    let backtrace = {
        let cls = cls.get_class();
        let id = cls.get_field_id(&new_br("backtrace"), &new_br("Ljava/lang/Object;"), false);
        Class::get_field_value(throwable.extract_ref(), id)
    };

    let v = {
        let rf = backtrace.extract_ref();
        oop::with_heap(|heap| {
            let desc = heap.get(rf);
            let guard = desc.read().unwrap();
            let ary = guard.v.extract_array();
            if index >= 0 && (index as usize) < ary.elements.len() {
                ary.elements[index as usize].clone()
            } else {
                Oop::Null
            }
        })
    };

    Ok(Some(v))
}
