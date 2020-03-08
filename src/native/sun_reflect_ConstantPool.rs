#![allow(non_snake_case)]

use crate::classfile::constant_pool;
use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{self, Oop};
use crate::runtime::JavaThread;
use crate::util;

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![new_fn(
        "getUTF8At0",
        "(Ljava/lang/Object;I)Ljava/lang/String;",
        Box::new(jvm_getUTF8At0),
    )]
}

fn jvm_getUTF8At0(jt: &mut JavaThread, _env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    let _this = args.get(0).unwrap();
    let cp_oop = args.get(1).unwrap();
    let index = {
        let index = args.get(2).unwrap();
        util::oop::extract_int(index)
    };

    let s = match cp_oop {
        Oop::Ref(rf) => {
            let rf = rf.read().unwrap();
            match &rf.v {
                oop::RefKind::Mirror(mirror) => {
                    let target = mirror.target.clone().unwrap();
                    let cls = target.read().unwrap();
                    match &cls.kind {
                        oop::class::ClassKind::Instance(inst) => {
                            let cp = &inst.class_file.cp;
                            constant_pool::get_utf8(cp, index as usize)
                        }
                        _ => unimplemented!(),
                    }
                }
                _ => unimplemented!(),
            }
        }
        _ => unreachable!(),
    };

    let r = match s {
        Some(s) => {
            let s = String::from_utf8_lossy(s.as_slice());
            error!("s = {}", s);
            util::oop::new_java_lang_string2(jt, s.as_ref())
        }
        None => oop::consts::get_null(),
    };

    Ok(Some(r))
}
