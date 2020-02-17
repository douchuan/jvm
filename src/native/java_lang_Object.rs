#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{Oop, OopDesc};
use crate::runtime::JavaThread;
use crate::types::OopRef;
use crate::util;

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![
        new_fn("registerNatives", "()V", Box::new(jvm_registerNatives)),
        new_fn("hashCode", "()I", Box::new(jvm_hashCode)),
        new_fn("clone", "()Ljava/lang/Object;", Box::new(jvm_clone)),
        new_fn("getClass", "()Ljava/lang/Class;", Box::new(jvm_getClass)),
        new_fn("notifyAll", "()V", Box::new(jvm_notifyAll)),
    ]
}

fn jvm_registerNatives(_jt: &mut JavaThread, _env: JNIEnv, _args: Vec<OopRef>) -> JNIResult {
    Ok(None)
}

pub fn jvm_hashCode(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let use_cache = true;
    let v = args.get(0).unwrap();

    if use_cache {
        let hash = { v.lock().unwrap().hash_code.clone() };
        match hash {
            Some(hash) => Ok(Some(OopDesc::new_int(hash))),
            None => {
                let hash = util::oop::hash_code(v.clone()) as i32;
                let mut v = v.lock().unwrap();
                v.hash_code = Some(hash);
                Ok(Some(OopDesc::new_int(hash)))
            }
        }
    } else {
        let hash = util::oop::hash_code(v.clone()) as i32;
        Ok(Some(OopDesc::new_int(hash)))
    }
}

fn jvm_clone(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    //    let java_lang_Cloneable = require_class3(None, b"java/lang/Cloneable").unwrap();
    let this_obj = args.get(0).unwrap();
    Ok(Some(this_obj.clone()))
}

fn jvm_getClass(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let v = args.get(0).unwrap();
    let mirror = {
        let v = v.lock().unwrap();
        match &v.v {
            Oop::Inst(inst) => {
                let cls = inst.class.lock().unwrap();
                cls.get_mirror()
            }
            Oop::Array(ary) => ary.class.lock().unwrap().get_mirror(),
            t => unimplemented!("t = {:?}", t),
        }
    };
    Ok(Some(mirror))
}

fn jvm_notifyAll(_jt: &mut JavaThread, _env: JNIEnv, _args: Vec<OopRef>) -> JNIResult {
    Ok(None)
}
