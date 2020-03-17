#![allow(non_snake_case)]

use class_parser::parser::parse as class_parse;
use crate::native;
use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{self, Class, Oop};
use crate::runtime::{self, JavaThread};
use crate::types::ClassRef;
use crate::util;
use std::sync::Arc;

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![new_fn(
        "defineClass0",
        "(Ljava/lang/ClassLoader;Ljava/lang/String;[BII)Ljava/lang/Class;",
        Box::new(jvm_defineClass0),
    )]
}

fn jvm_defineClass0(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    let _loader = args.get(0).unwrap();
    let name = args.get(1).unwrap();
    let name = util::oop::extract_str(name);
    let b = args.get(2).unwrap();
    let off = args.get(3).unwrap();
    let off = util::oop::extract_int(off);
    let len = args.get(4).unwrap();
    let len = util::oop::extract_int(len);

    let name = name.replace(".", "/");

    //parse bytes => class, put in sys_dic
    let class = do_parse_class(b, off as usize, len as usize);
    runtime::sys_dic_put(name.as_bytes(), class.clone());
    {
        let this_ref = class.clone();
        let mut cls = class.write().unwrap();
        cls.set_class_state(oop::class::State::Loaded);
        cls.link_class(this_ref);
    }
    native::java_lang_Class::create_mirror(class.clone());

    let v = Oop::new_mirror(class);
    Ok(Some(v))
}

fn do_parse_class(v: &Oop, off: usize, len: usize) -> ClassRef {
    match v {
        Oop::Ref(rf) => {
            let rf = rf.read().unwrap();
            match &rf.v {
                oop::RefKind::TypeArray(ary) => {
                    match ary {
                        oop::TypeArrayDesc::Byte(ary) => {
                            match class_parse(&ary[off..(off + len)]) {
                                Ok(r) => {
                                    let cfr = Arc::new(Box::new(r.1));
                                    //fixme: setup classloader
                                    let class = Class::new_class(cfr, None);
                                    new_sync_ref!(class)
                                }
                                Err(e) => unreachable!("e = {:?}", e),
                            }
                        }
                        _ => unreachable!(),
                    }
                }
                _ => unreachable!(),
            }
        }
        _ => unreachable!(),
    }
}
