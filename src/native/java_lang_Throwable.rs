#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{self, Oop, OopDesc};
use crate::runtime::{self, require_class3, JavaThread};
use crate::types::OopRef;
use crate::util;

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![
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

fn jvm_fillInStackTrace(jt: &mut JavaThread, _env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let elm_cls = oop::class::load_and_init(jt, b"java/lang/StackTraceElement");
    let ary_cls = require_class3(None, b"[Ljava/lang/StackTraceElement;").unwrap();

    let throwable_oop = args.get(0).unwrap();
    let callers = jt.callers.clone();

    let mut traces = Vec::new();
    for mir in callers.iter().rev() {
        let cls_name = { mir.method.class.lock().unwrap().name.clone() };
        let method_name = mir.method.name.clone();
        let src_file = mir.method.src_file.clone();
        let src_file = match src_file {
            Some(name) => util::oop::new_java_lang_string3(jt, name.as_slice()),
            None => util::oop::new_java_lang_string2(jt, ""),
        };

        let elm = OopDesc::new_inst(elm_cls.clone());
        let args = vec![
            elm.clone(),
            util::oop::new_java_lang_string3(jt, cls_name.as_slice()),
            util::oop::new_java_lang_string3(jt, method_name.as_slice()),
            src_file,
            OopDesc::new_int(0),
        ];
        runtime::java_call::invoke_ctor(
            jt,
            elm_cls.clone(),
            b"(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;I)V",
            args,
        );

        traces.push(elm);
    }

    let stack_trace_ary = OopDesc::new_ref_ary2(ary_cls, traces);
    let throwable_cls = require_class3(None, b"java/lang/Throwable").unwrap();
    {
        let cls = throwable_cls.lock().unwrap();
        let id = cls.get_field_id(b"stackTrace", b"[Ljava/lang/StackTraceElement;", false);
        cls.put_field_value(throwable_oop.clone(), id, oop::consts::get_null());
        let id = cls.get_field_id(b"backtrace", b"Ljava/lang/Object;", false);
        cls.put_field_value(throwable_oop.clone(), id, stack_trace_ary);
    }

    Ok(Some(throwable_oop.clone()))
}

fn jvm_getStackTraceDepth(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let throwable = args.get(0).unwrap();
    let cls = {
        let v = throwable.lock().unwrap();
        match &v.v {
            Oop::Inst(inst) => inst.class.clone(),
            _ => unreachable!(),
        }
    };
    let backtrace = {
        let cls = cls.lock().unwrap();
        let id = cls.get_field_id(b"backtrace", b"Ljava/lang/Object;", false);
        cls.get_field_value(throwable.clone(), id)
    };

    let v = backtrace.lock().unwrap();
    let v = match &v.v {
        Oop::Array(ary) => {
            error!("backtrace len = {}", ary.elements.len());
            OopDesc::new_int(ary.elements.len() as i32)
        }
        Oop::Null => OopDesc::new_int(0),
        _ => unreachable!(),
    };

    Ok(Some(v))
}

fn jvm_getStackTraceElement(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let throwable = args.get(0).unwrap();
    let index = {
        let v = args.get(1).unwrap();
        let v = v.lock().unwrap();
        match v.v {
            Oop::Int(v) => v,
            _ => unreachable!(),
        }
    };
    let cls = {
        let v = throwable.lock().unwrap();
        match &v.v {
            Oop::Inst(inst) => inst.class.clone(),
            _ => unreachable!(),
        }
    };
    let backtrace = {
        let cls = cls.lock().unwrap();
        let id = cls.get_field_id(b"backtrace", b"Ljava/lang/Object;", false);
        cls.get_field_value(throwable.clone(), id)
    };

    let v = backtrace.lock().unwrap();
    let v = match &v.v {
        Oop::Array(ary) => {
            if index >= 0 && (index as usize) < ary.elements.len() {
                ary.elements[index as usize].clone()
            } else {
                oop::consts::get_null()
            }
        }
        _ => unreachable!(),
    };

    Ok(Some(v))
}
