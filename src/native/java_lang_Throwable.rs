#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{self, Oop, OopDesc, OopRef};
use crate::runtime::{self, require_class3, JavaThread};
use crate::util;
use std::sync::{Arc, Mutex};

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![new_fn(
        "fillInStackTrace",
        "(I)Ljava/lang/Throwable;",
        Box::new(jvm_fillInStackTrace),
    )]
}

fn jvm_fillInStackTrace(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let elm_cls = oop::class::load_and_init(jt, b"java/lang/StackTraceElement");
    let ary_cls = require_class3(None, b"[Ljava/lang/StackTraceElement;").unwrap();

    let throwable_oop = args.get(0).unwrap();
    let mut elms = Vec::new();
    let frames = jt.frames.clone();

    for it in frames.iter() {
        match it.try_lock() {
            Ok(frame) => {
                let mir = frame.mir.clone();
                let pc = frame.pc;

                //todo: native frame
                let line_number = {
                    let pc = pc as u16;
                    match mir.method.lnt.get(&pc) {
                        Some(&v) => v as i32,
                        _ => -2,
                    }
                };

                let cls_name = { mir.method.class.lock().unwrap().name.clone() };
                let method_name = mir.method.name.clone();
                //todo: class source file name
                let src_file = Vec::from("xxx");
                let src_file = new_ref!(src_file);

                let elm = OopDesc::new_inst(elm_cls.clone());
                let args = vec![
                    elm.clone(),
                    OopDesc::new_str(cls_name),
                    OopDesc::new_str(method_name),
                    OopDesc::new_str(src_file),
                    OopDesc::new_int(line_number),
                ];
                runtime::java_call::invoke_ctor(
                    jt,
                    elm_cls.clone(),
                    b"(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;I)V",
                    args,
                );

                elms.push(elm);
            }
            _ => break,
        }
    }

    let stack_trace_ary = OopDesc::new_ary2(ary_cls, elms);
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
