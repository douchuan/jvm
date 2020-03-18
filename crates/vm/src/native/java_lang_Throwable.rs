#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{self, Oop};
use crate::runtime::{self, require_class3, JavaThread};
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

fn jvm_fillInStackTrace(jt: &mut JavaThread, _env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    let elm_cls = oop::class::load_and_init(jt, b"java/lang/StackTraceElement");
    let ary_cls = require_class3(None, b"[Ljava/lang/StackTraceElement;").unwrap();

    let throwable_oop = args.get(0).unwrap();
    let mut backtrace = Vec::with_capacity(jt.frames.len());

    let mut found_ex_here = false;
    for it in jt.frames.iter() {
        let ex_here = {
            let it = it.try_read().unwrap();
            let area = it.area.borrow();
            area.ex_here
        };

        backtrace.push(it.clone());

        if ex_here {
            found_ex_here = true;
            break;
        }
    }

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
            let pc = { caller.area.borrow().pc };
            (caller.mir.clone(), pc)
        };

        let cls_name = { mir.method.class.read().unwrap().name.clone() };
        let cls_name = String::from_utf8_lossy(cls_name.as_slice()).replace("/", ".");
        let method_name = mir.method.name.clone();
        let method_name = unsafe { std::str::from_utf8_unchecked(method_name.as_slice()) };
        let src_file = {
            let cls = mir.method.class.read().unwrap();
            cls.get_source_file()
        };
        let src_file = match src_file {
            Some(name) => {
                let name = unsafe { std::str::from_utf8_unchecked(name.as_slice()) };
                util::oop::new_java_lang_string2(jt, name)
            }
            None => util::oop::new_java_lang_string2(jt, ""),
        };
        let line_num = mir.method.get_line_num((pc - 1) as u16);

        let elm = Oop::new_inst(elm_cls.clone());
        let args = vec![
            elm.clone(),
            util::oop::new_java_lang_string2(jt, &cls_name),
            util::oop::new_java_lang_string2(jt, &method_name),
            src_file,
            Oop::new_int(line_num),
        ];
        runtime::java_call::invoke_ctor(
            jt,
            elm_cls.clone(),
            b"(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;I)V",
            args,
        );

        traces.push(elm);
    }

    let stack_trace_ary = Oop::new_ref_ary2(ary_cls, traces);
    let throwable_cls = require_class3(None, b"java/lang/Throwable").unwrap();
    {
        let cls = throwable_cls.read().unwrap();
        let id = cls.get_field_id(b"stackTrace", b"[Ljava/lang/StackTraceElement;", false);
        cls.put_field_value(throwable_oop.clone(), id, oop::consts::get_null());
        let id = cls.get_field_id(b"backtrace", b"Ljava/lang/Object;", false);
        cls.put_field_value(throwable_oop.clone(), id, stack_trace_ary);
    }

    Ok(Some(throwable_oop.clone()))
}

fn jvm_getStackTraceDepth(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    let throwable = args.get(0).unwrap();
    let cls = {
        let throwable = util::oop::extract_ref(throwable);
        let v = throwable.read().unwrap();
        match &v.v {
            oop::RefKind::Inst(inst) => inst.class.clone(),
            _ => unreachable!(),
        }
    };
    let backtrace = {
        let cls = cls.read().unwrap();
        let id = cls.get_field_id(b"backtrace", b"Ljava/lang/Object;", false);
        cls.get_field_value(throwable, id)
    };

    let v = match backtrace {
        Oop::Null => Oop::new_int(0),
        Oop::Ref(rf) => {
            let rf = rf.read().unwrap();
            match &rf.v {
                oop::RefKind::Array(ary) => Oop::new_int(ary.elements.len() as i32),
                _ => unreachable!(),
            }
        }
        _ => unreachable!(),
    };

    Ok(Some(v))
}

fn jvm_getStackTraceElement(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    let throwable = args.get(0).unwrap();
    let index = util::oop::extract_int(args.get(1).unwrap());
    let cls = {
        let throwable = util::oop::extract_ref(throwable);
        let v = throwable.read().unwrap();
        match &v.v {
            oop::RefKind::Inst(inst) => inst.class.clone(),
            _ => unreachable!(),
        }
    };
    let backtrace = {
        let cls = cls.read().unwrap();
        let id = cls.get_field_id(b"backtrace", b"Ljava/lang/Object;", false);
        cls.get_field_value(throwable, id)
    };

    let backtrace = util::oop::extract_ref(&backtrace);
    let v = backtrace.read().unwrap();
    let v = match &v.v {
        oop::RefKind::Array(ary) => {
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
