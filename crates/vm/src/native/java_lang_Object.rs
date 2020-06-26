#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{self, Oop};
use std::time::Duration;

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![
        new_fn("registerNatives", "()V", Box::new(jvm_registerNatives)),
        new_fn("hashCode", "()I", Box::new(jvm_hashCode)),
        new_fn("clone", "()Ljava/lang/Object;", Box::new(jvm_clone)),
        new_fn("getClass", "()Ljava/lang/Class;", Box::new(jvm_getClass)),
        new_fn("notifyAll", "()V", Box::new(jvm_notifyAll)),
        new_fn("wait", "(J)V", Box::new(jvm_wait)),
    ]
}

fn jvm_registerNatives(_env: JNIEnv, _args: &Vec<Oop>) -> JNIResult {
    Ok(None)
}

pub fn jvm_hashCode(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let v = args.get(0).unwrap();
    let v = v.hash_code();
    let v = Oop::new_int(v);
    Ok(Some(v))
}

fn jvm_clone(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    //    let java_lang_Cloneable = require_class3(None, b"java/lang/Cloneable").unwrap();
    let this_obj = args.get(0).unwrap();
    Ok(Some(this_obj.clone()))
}

fn jvm_getClass(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let v = args.get(0).unwrap();
    let mirror = {
        let rf = v.extract_ref();
        let ptr = rf.get_raw_ptr();
        unsafe {
            match &(*ptr).v {
                oop::RefKind::Inst(inst) => {
                    let cls = inst.class.get_class();
                    cls.get_mirror()
                }
                oop::RefKind::Array(ary) => ary.class.get_class().get_mirror(),
                oop::RefKind::Mirror(_mirror) => v.clone(),
                t => unimplemented!("t = {:?}", t),
            }
        }
    };
    Ok(Some(mirror))
}

fn jvm_notifyAll(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let this = args.get(0).unwrap();
    let rf = this.extract_ref();
    rf.notify_all();
    Ok(None)
}

fn jvm_wait(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let this = args.get(0).unwrap();
    let millis = args.get(1).unwrap().extract_long();

    let rf = this.extract_ref();
    if millis == 0 {
        rf.wait();
    } else {
        rf.wait_timeout(Duration::from_millis(millis as u64));
    }

    Ok(None)
}
