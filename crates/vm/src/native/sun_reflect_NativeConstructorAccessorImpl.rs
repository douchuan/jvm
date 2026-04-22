#![allow(non_snake_case)]

use crate::native::{common, new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{self, Oop};
use crate::{new_br, runtime};
use tracing::{debug, error, info, trace, warn};

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![new_fn(
        "newInstance0",
        "(Ljava/lang/reflect/Constructor;[Ljava/lang/Object;)Ljava/lang/Object;",
        Box::new(jvm_newInstance0),
    )]
}

fn jvm_newInstance0(_env: JNIEnv, args: &[Oop]) -> JNIResult {
    let ctor = args.get(0).unwrap();
    let arguments = args.get(1).unwrap();

    let clazz = common::reflect::get_Constructor_clazz(ctor);
    let target_cls = { Oop::mirror_target(clazz.extract_ref()).unwrap() };

    let signature = common::reflect::get_Constructor_signature(ctor);

    let cls = target_cls.get_class();
    let name = unsafe { std::str::from_utf8_unchecked(cls.name.as_slice()) };
    info!("newInstance0 {}:{}", name, signature);

    let mut ctor_args = Vec::new();
    {
        match arguments {
            Oop::Null => (),
            Oop::Ref(slot_id) => {
                let args = oop::with_heap(|heap| {
                    let desc = heap.get(*slot_id);
                    let guard = desc.read().unwrap();
                    guard.v.extract_array().elements.clone()
                });
                ctor_args.extend_from_slice(&args);
            }
            _ => unreachable!(),
        }
    }

    let oop = Oop::new_inst(target_cls.clone());
    ctor_args.insert(0, oop.clone());
    runtime::invoke::invoke_ctor(target_cls, new_br(signature.as_str()), ctor_args);

    Ok(Some(oop))
}
