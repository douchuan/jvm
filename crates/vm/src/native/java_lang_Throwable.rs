#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{self, Class, Oop};
use crate::runtime::{self, require_class3};
use crate::{new_br, util};
use std::sync::atomic::Ordering;

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

fn jvm_fillInStackTrace(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let jt = runtime::thread::current_java_thread();

    let elm_cls = oop::class::load_and_init(b"java/lang/StackTraceElement");
    let ary_cls = require_class3(None, b"[Ljava/lang/StackTraceElement;").unwrap();

    let throwable_oop = args.get(0).unwrap();
    let mut backtrace = Vec::with_capacity(jt.read().unwrap().frames.len());

    let mut found_ex_here = false;
    let jth = jt.read().unwrap();
    for it in jth.frames.iter() {
        let ex_here = {
            let it = it.try_read().unwrap();
            it.ex_here.load(Ordering::Relaxed)
        };

        backtrace.push(it.clone());

        if ex_here {
            found_ex_here = true;
            break;
        }
    }
    drop(jth);

    /*
       todo: how handle throw better?

    if no ex_here found, it's:
      throw new AnnotationFormatError("Unexpected end of annotations.");

    new Throwable
      Throwable.fillInStackTrace invoked, and come here

    there are stacktraces for build 'Throwable' obj, not necessary for user, need discard

    Exception in thread "main" java.lang.annotation.AnnotationFormatError: Unexpected end of annotations.
       at java.lang.Throwable.fillInStackTrace(Throwable.java)
       at java.lang.Throwable.fillInStackTrace(Throwable.java:783)
       at java.lang.Throwable.<init>(Throwable.java:265)
       at java.lang.Error.<init>(Error.java:70)
       */
    if !found_ex_here {
        backtrace.pop();
        backtrace.pop();
        backtrace.pop();
        backtrace.pop();
    }

    let mut traces = Vec::new();
    for caller in backtrace.iter().rev() {
        let (mir, pc) = {
            let caller = caller.try_read().unwrap();
            let pc = caller.pc.load(Ordering::Relaxed);
            (caller.mir.clone(), pc)
        };

        let cls = mir.method.class.get_class();
        let cls_name = unsafe { std::str::from_utf8_unchecked(cls.name.as_slice()) };
        let cls_name = cls_name.replace("/", ".");
        let method_name = unsafe { std::str::from_utf8_unchecked(mir.method.name.as_slice()) };
        let src_file = {
            let cls = mir.method.class.get_class();
            cls.get_source_file()
        };
        let src_file = match src_file {
            Some(name) => {
                let name = unsafe { std::str::from_utf8_unchecked(name.as_slice()) };
                util::oop::new_java_lang_string2(name)
            }
            None => util::oop::new_java_lang_string2(""),
        };
        let line_num = mir.method.get_line_num((pc - 1) as u16);

        let elm = Oop::new_inst(elm_cls.clone());
        let args = vec![
            elm.clone(),
            util::oop::new_java_lang_string2(&cls_name),
            util::oop::new_java_lang_string2(method_name),
            src_file,
            Oop::new_int(line_num),
        ];
        runtime::invoke::invoke_ctor(
            elm_cls.clone(),
            new_br("(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;I)V"),
            args,
        );

        traces.push(elm);
    }

    let stack_trace_ary = Oop::new_ref_ary2(ary_cls, traces);
    let throwable_cls = require_class3(None, b"java/lang/Throwable").unwrap();
    {
        let cls = throwable_cls.get_class();
        let id = cls.get_field_id(
            new_br("stackTrace"),
            new_br("[Ljava/lang/StackTraceElement;"),
            false,
        );
        Class::put_field_value(throwable_oop.extract_ref(), id, oop::consts::get_null());
        let id = cls.get_field_id(new_br("backtrace"), new_br("Ljava/lang/Object;"), false);
        Class::put_field_value(throwable_oop.extract_ref(), id, stack_trace_ary);
    }

    Ok(Some(throwable_oop.clone()))
}

fn jvm_getStackTraceDepth(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let throwable = args.get(0).unwrap();
    let cls = {
        let rf = throwable.extract_ref();
        let inst = rf.extract_inst();
        inst.class.clone()
    };
    let backtrace = {
        let cls = cls.get_class();
        let id = cls.get_field_id(new_br("backtrace"), new_br("Ljava/lang/Object;"), false);
        Class::get_field_value(throwable.extract_ref(), id)
    };

    let v = match backtrace {
        Oop::Null => Oop::new_int(0),
        Oop::Ref(rf) => {
            let ary = rf.extract_array();
            let len = ary.elements.len();
            Oop::new_int(len as i32)
        }
        _ => unreachable!(),
    };

    Ok(Some(v))
}

fn jvm_getStackTraceElement(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let throwable = args.get(0).unwrap();
    let index = args.get(1).unwrap().extract_int();
    let cls = {
        let rf = throwable.extract_ref();
        let inst = rf.extract_inst();
        inst.class.clone()
    };
    let backtrace = {
        let cls = cls.get_class();
        let id = cls.get_field_id(new_br("backtrace"), new_br("Ljava/lang/Object;"), false);
        Class::get_field_value(throwable.extract_ref(), id)
    };

    let v = {
        let rf = backtrace.extract_ref();
        let ary = rf.extract_array();
        if index >= 0 && (index as usize) < ary.elements.len() {
            ary.elements[index as usize].clone()
        } else {
            oop::consts::get_null()
        }
    };

    Ok(Some(v))
}
