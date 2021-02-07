#![allow(non_snake_case)]

use crate::native;
use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::class::ClassPtr;
use crate::oop::{self, Class, Oop, OopPtr};
use crate::runtime;
use crate::types::ClassRef;
use class_parser::parse_class;
use std::sync::Arc;

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![new_fn(
        "defineClass0",
        "(Ljava/lang/ClassLoader;Ljava/lang/String;[BII)Ljava/lang/Class;",
        Box::new(jvm_defineClass0),
    )]
}

fn jvm_defineClass0(_env: JNIEnv, args: &[Oop]) -> JNIResult {
    let _loader = args.get(0).unwrap();
    let name = args.get(1).unwrap();
    let name = OopPtr::java_lang_string(name.extract_ref());
    let b = args.get(2).unwrap();
    let off = args.get(3).unwrap().extract_int();
    let len = args.get(4).unwrap().extract_int();

    let name = name.replace(".", "/");

    //parse bytes => class, put in sys_dic
    let class = do_parse_class(b, off as usize, len as usize);
    runtime::sys_dic_put(name.as_bytes(), class.clone());
    {
        let this_ref = class.clone();
        let cls = class.get_mut_class();
        cls.set_class_state(oop::class::State::Loaded);
        cls.link_class(this_ref);
    }
    native::java_lang_Class::create_mirror(class.clone());

    let v = Oop::new_mirror(class);
    Ok(Some(v))
}

fn do_parse_class(v: &Oop, off: usize, len: usize) -> ClassRef {
    let rf = v.extract_ref();
    let ary = rf.extract_type_array();
    let ary = ary.extract_bytes();
    match parse_class(&ary[off..(off + len)]) {
        Ok(r) => {
            let cfr = Arc::new(Box::new(r.1));
            //fixme: setup classloader
            let class = Class::new_class(cfr, None);
            ClassPtr::new(class)
        }
        Err(e) => unreachable!("e = {:?}", e),
    }
}
