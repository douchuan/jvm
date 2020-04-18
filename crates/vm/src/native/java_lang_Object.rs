#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{self, Oop};
use crate::runtime::JavaThread;
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

fn jvm_registerNatives(_jt: &mut JavaThread, _env: JNIEnv, _args: Vec<Oop>) -> JNIResult {
    Ok(None)
}

pub fn jvm_hashCode(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    let v = args.get(0).unwrap();
    let v = match v {
        Oop::Null => Oop::new_int(0),
        Oop::Ref(rf) => {
            let hash =  rf.read().unwrap().hash_code.clone() ;
            match hash {
                Some(hash) => Oop::new_int(hash),
                None => {
                    let hash = util::oop::hash_code(v);
                    let mut v = rf.write().unwrap();
                    v.hash_code = Some(hash);
                    Oop::new_int(hash)
                }
            }
        }
        _ => unreachable!(),
    };

    Ok(Some(v))
}

fn jvm_clone(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    //    let java_lang_Cloneable = require_class3(None, b"java/lang/Cloneable").unwrap();
    let this_obj = args.get(0).unwrap();
    Ok(Some(this_obj.clone()))
}

fn jvm_getClass(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    let v = args.get(0).unwrap();
    let mirror = {
        let rf = util::oop::extract_ref(v);
        let rf = rf.read().unwrap();
        match &rf.v {
            oop::RefKind::Inst(inst) => {
                let cls = inst.class.read().unwrap();
                cls.get_mirror()
            }
            oop::RefKind::Array(ary) => ary.class.read().unwrap().get_mirror(),
            oop::RefKind::Mirror(_mirror) => {
                v.clone()

                /*
                let cls = mirror.target.clone().unwrap();
                let cls = cls.lock().unwrap();
                let name = String::from_utf8_lossy(cls.name.as_slice());
                error!("target cls = {}", name);
                cls.get_mirror()
                */
            }
            t => unimplemented!("t = {:?}", t),
        }
    };
    Ok(Some(mirror))
}

fn jvm_notifyAll(_jt: &mut JavaThread, _env: JNIEnv, _args: Vec<Oop>) -> JNIResult {
    Ok(None)
}
