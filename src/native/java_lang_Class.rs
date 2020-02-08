#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{self, ClassRef, Oop, OopDesc, OopRef, ValueType};
use crate::runtime::{require_class3, JavaThread};
use crate::util;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![
        new_fn("registerNatives", "()V", Box::new(jvm_register_natives)),
        new_fn(
            "desiredAssertionStatus0",
            "(Ljava/lang/Class;)Z",
            Box::new(jvm_desiredAssertionStatus0),
        ),
        new_fn(
            "getPrimitiveClass",
            "(Ljava/lang/String;)Ljava/lang/Class;",
            Box::new(jvm_getPrimitiveClass),
        ),
    ]
}

#[derive(Copy, Clone, PartialEq)]
enum ClassMirrorState {
    NotFixed,
    Fixed
}

lazy_static! {
    static ref MIRROR_STATE: Mutex<ClassMirrorState> = {
        Mutex::new(ClassMirrorState::NotFixed)
    };

    static ref PRIM_MIRROS: Mutex<HashMap<String, OopRef>> = {
        let hm = HashMap::new();
        Mutex::new(hm)
    };
    static ref SIGNATURE_DIC: HashMap<&'static str, &'static str> = {
        let dic: HashMap<&'static str, &'static str> = [
            ("byte", "B"),
            ("boolean", "Z"),
            ("char", "C"),
            ("short", "S"),
            ("int", "I"),
            ("float", "F"),
            ("long", "J"),
            ("double", "D"),
            ("void", "V"),
        ]
        .iter()
        .cloned()
        .collect();

        dic
    };
    static ref DELAYED_MIRROS: Mutex<Vec<String>> = {
        let v = vec![ "I", "Z", "B", "C", "S", "F", "J", "D", "V", "[I", "[Z", "[B", "[C", "[S", "[F", "[J", "[D",];
        let v: Vec<String> = v.iter().map(|it| it.to_string()).collect();
        Mutex::new(v)
    };
    static ref DELAYED_ARY_MIRROS: Mutex<Vec<ClassRef>> = {
        let v = vec![];
        Mutex::new(v)
    };
}

pub fn init() {
    lazy_static::initialize(&MIRROR_STATE);
    lazy_static::initialize(&SIGNATURE_DIC);
    lazy_static::initialize(&PRIM_MIRROS);
    lazy_static::initialize(&DELAYED_MIRROS);
    lazy_static::initialize(&DELAYED_ARY_MIRROS);
}

pub fn create_mirror(cls: ClassRef) {
    let is_fixed = util::sync_call_ctx(&MIRROR_STATE, |s| {
        *s == ClassMirrorState::Fixed
    });

    if is_fixed {
        let mirror = OopDesc::new_mirror(cls.clone());
        let mut cls = cls.lock().unwrap();
        trace!("mirror created: {}", String::from_utf8_lossy(cls.name.as_slice()));
        cls.set_mirror(mirror);
    } else {
        let cls_back = cls.clone();
        let cls = cls.lock().unwrap();
        let name = String::from_utf8_lossy(cls.name.as_slice()).to_string();
        warn!("mirror create delayed: {}", name);
        match cls.kind {
            oop::class::ClassKind::Instance(_) => {
                util::sync_call_ctx(&DELAYED_MIRROS, |mirros| {
                    mirros.push(name);
                });
            }
            _ => {
                util::sync_call_ctx(&DELAYED_ARY_MIRROS, |mirros| {
                    mirros.push(cls_back);
                });
            }
        }
    }
}

pub fn create_delayed_mirrors() {
    let names: Vec<String> = {
        let mirros = DELAYED_MIRROS.lock().unwrap();
        mirros.clone()
    };

    util::sync_call_ctx(&MIRROR_STATE, |s| {
        *s = ClassMirrorState::Fixed;
    });

    for name in names {
        if name.len() > 2 {
            //java.lang.XXX
            let target = require_class3(None, name.as_bytes()).unwrap();
            create_mirror(target);
        } else {
            let is_prim_ary = name.as_bytes()[0] == b'[';
            let vt = if is_prim_ary {
                ValueType::from(&name.as_bytes()[1])
            } else {
                ValueType::from(&name.as_bytes()[0])
            };

            let mirror = OopDesc::new_prim_mirror(vt);
            if is_prim_ary {
                let target = require_class3(None, name.as_bytes()).unwrap();
                let mut mirror = mirror.lock().unwrap();
                match &mut mirror.v {
                    Oop::Mirror(mirror) => {
                        mirror.target = Some(target);
                    }
                    _ => unreachable!(),
                }
            }

            util::sync_call_ctx(&PRIM_MIRROS, |mirrors| {
                mirrors.insert(name.to_string(), mirror);
            });
        }
    }
}

pub fn create_delayed_ary_mirrors() {
    let classes: Vec<ClassRef> = {
        let mirros = DELAYED_ARY_MIRROS.lock().unwrap();
        mirros.clone()
    };

    for cls in classes {
        let value_type = {
            let cls = cls.lock().unwrap();
            match &cls.kind {
                oop::class::ClassKind::ObjectArray(obj_ary) => obj_ary.value_type,
                oop::class::ClassKind::TypeArray(typ_ary) => typ_ary.value_type,
                _ => unreachable!()
            }
        };
        let mirror = OopDesc::new_ary_mirror(cls.clone(), value_type);
        let mut cls = cls.lock().unwrap();
        cls.set_mirror(mirror);
    }
}

fn jvm_register_natives(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    Ok(None)
}

fn jvm_desiredAssertionStatus0(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    Ok(Some(OopDesc::new_int(0)))
}

fn jvm_getPrimitiveClass(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let v = args.get(0).unwrap();

    let v = {
        let v = v.lock().unwrap();
        match &v.v {
            Oop::Str(s) => s.clone(),
            _ => unreachable!(),
        }
    };

    let s = std::str::from_utf8(v.as_slice()).unwrap();
    match SIGNATURE_DIC.get(s) {
        Some(&s) => {
            //todo: avoid mutex lock, it's only read
            util::sync_call(&PRIM_MIRROS, |mirros| {
                Ok(mirros.get(s).map(|it| it.clone()))
            })
        }
        _ => unreachable!("Unknown primitive type: {}", s),
    }
}
