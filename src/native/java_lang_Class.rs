#![allow(non_snake_case)]

use crate::classfile::{self, access_flags as acc};
use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{self, ClassRef, FieldIdRef, Oop, OopDesc, OopRef, ValueType};
use crate::runtime::{self, require_class3, Exception, JavaThread};
use crate::util;
use nix::errno::errno;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};

pub fn get_primitive_class_mirror(key: &str) -> Option<OopRef> {
    //todo: avoid mutex lock, it's only read
    util::sync_call(&PRIM_MIRROS, |mirros| mirros.get(key).map(|it| it.clone()))
}

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![
        new_fn("registerNatives", "()V", Box::new(jvm_registerNatives)),
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
        new_fn(
            "getDeclaredFields0",
            "(Z)[Ljava/lang/reflect/Field;",
            Box::new(jvm_getDeclaredFields0),
        ),
        new_fn("getName0", "()Ljava/lang/String;", Box::new(jvm_getName0)),
        new_fn(
            "forName0",
            "(Ljava/lang/String;ZLjava/lang/ClassLoader;Ljava/lang/Class;)Ljava/lang/Class;",
            Box::new(jvm_forName0),
        ),
        new_fn("isPrimitive", "()Z", Box::new(jvm_isPrimitive)),
        new_fn(
            "isAssignableFrom",
            "(Ljava/lang/Class;)Z",
            Box::new(jvm_isAssignableFrom),
        ),
        new_fn("isInterface", "()Z", Box::new(jvm_isInterface)),
        new_fn(
            "getDeclaredConstructors0",
            "(Z)[Ljava/lang/reflect/Constructor;",
            Box::new(jvm_getDeclaredConstructors0),
        ),
        new_fn("getModifiers", "()I", Box::new(jvm_getModifiers)),
        new_fn(
            "getSuperclass",
            "()Ljava/lang/Class;",
            Box::new(jvm_getSuperclass),
        ),
    ]
}

#[derive(Copy, Clone, PartialEq)]
enum ClassMirrorState {
    NotFixed,
    Fixed,
}

lazy_static! {
    static ref MIRROR_STATE: Mutex<ClassMirrorState> = { Mutex::new(ClassMirrorState::NotFixed) };
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
        let v = vec![
            "I", "Z", "B", "C", "S", "F", "J", "D", "V", "[I", "[Z", "[B", "[C", "[S", "[F", "[J",
            "[D",
        ];
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
    let is_fixed = util::sync_call_ctx(&MIRROR_STATE, |s| *s == ClassMirrorState::Fixed);

    if is_fixed {
        let mirror = OopDesc::new_mirror(cls.clone());
        let mut cls = cls.lock().unwrap();
        trace!(
            "mirror created: {}",
            String::from_utf8_lossy(cls.name.as_slice())
        );
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

/*
called after 'java/lang/Class' inited in init_vm.rs
*/
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

                //set mirror's target
                {
                    let mut mirror = mirror.lock().unwrap();
                    match &mut mirror.v {
                        Oop::Mirror(mirror) => {
                            mirror.target = Some(target.clone());
                        }
                        _ => unreachable!(),
                    }
                }

                let mut cls = target.lock().unwrap();
                //                warn!("set_mirror name={}", String::from_utf8_lossy(cls.name.as_slice()));
                cls.set_mirror(mirror.clone());
            }

            util::sync_call_ctx(&PRIM_MIRROS, |mirrors| {
                mirrors.insert(name.to_string(), mirror);
            });
        }
    }
}

/*
called after 'java/lang/Class' inited in init_vm.rs
*/
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
                _ => unreachable!(),
            }
        };
        let mirror = OopDesc::new_ary_mirror(cls.clone(), value_type);
        let mut cls = cls.lock().unwrap();
        cls.set_mirror(mirror);
    }
}

fn jvm_registerNatives(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
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
        Some(&s) => Ok(get_primitive_class_mirror(s)),
        _ => unreachable!("Unknown primitive type: {}", s),
    }
}

fn jvm_getDeclaredFields0(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    //parse args
    let mirror_target = {
        let arg0 = args.get(0).unwrap();
        let arg0 = arg0.lock().unwrap();
        match &arg0.v {
            Oop::Mirror(mirror) => mirror.target.clone().unwrap(),
            _ => unreachable!(),
        }
    };

    let public_only = {
        let arg1 = args.get(1).unwrap();
        let arg1 = arg1.lock().unwrap();
        match arg1.v {
            Oop::Int(v) => v == 1,
            _ => unreachable!(),
        }
    };

    //fixme: super fields
    //obtain inst&static fields
    let (inst_fields, static_fields) = {
        let cls = mirror_target.lock().unwrap();
        match &cls.kind {
            oop::class::ClassKind::Instance(inst) => {
                (inst.inst_fields.clone(), inst.static_fields.clone())
            }
            _ => unreachable!(),
        }
    };

    //build fields ary
    let mut fields = Vec::new();
    for (_, it) in inst_fields {
        if public_only && !it.field.is_public() {
            continue;
        }

        let v = runtime::reflect::new_field(jt, it);
        fields.push(v);
    }

    for (_, it) in static_fields {
        if public_only && !it.field.is_public() {
            continue;
        }

        let v = runtime::reflect::new_field(jt, it);
        fields.push(v);
    }

    //build oop field ar
    let ary_cls = require_class3(None, b"[Ljava/lang/reflect/Field;").unwrap();
    Ok(Some(OopDesc::new_ary2(ary_cls, fields)))
}

fn jvm_getName0(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let target = {
        let arg0 = args.get(0).unwrap();
        let arg0 = arg0.lock().unwrap();
        match &arg0.v {
            Oop::Mirror(mirror) => mirror.target.clone().unwrap(),
            _ => unreachable!(),
        }
    };
    let name = {
        let cls = target.lock().unwrap();
        cls.name.clone()
    };

    let name = String::from_utf8_lossy(name.as_slice());
    let name = name.replace("/", ".");
    let v = Vec::from(name.as_bytes());
    let v = new_ref!(v);
    Ok(Some(OopDesc::new_str(v)))
}

fn jvm_forName0(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let java_name = {
        let arg0 = args.get(0).unwrap();
        let arg0 = arg0.lock().unwrap();
        match &arg0.v {
            Oop::Str(s) => s.clone(),
            _ => unreachable!(),
        }
    };
    let initialize = {
        let arg1 = args.get(1).unwrap();
        let arg1 = arg1.lock().unwrap();
        match arg1.v {
            Oop::Int(v) => v != 0,
            _ => unreachable!(),
        }
    };
    let java_cls_loader = args.get(2).unwrap();
    {
        let v = java_cls_loader.lock().unwrap();
        match &v.v {
            Oop::Null => (),
            _ => unimplemented!("app class loader, unimpl"),
        }
    }

    let caller_mirror = args.get(3).unwrap();

    let name = String::from_utf8_lossy(java_name.as_slice());
    let name = name.replace(".", "/");
    info!("forName0: {}", name);
    let cls = require_class3(None, name.as_bytes());

    match cls {
        Some(cls) => {
            {
                let mut cls = cls.lock().unwrap();
                cls.init_class(jt);
                //                trace!("finish init_class: {}", String::from_utf8_lossy(*c));
            }
            oop::class::init_class_fully(jt, cls.clone());

            let mirror = { cls.lock().unwrap().get_mirror() };

            Ok(Some(mirror))
        }
        None => {
            let cls_name = Vec::from(classfile::consts::J_NPE);
            let exception = Exception {
                cls_name: new_ref!(cls_name),
                msg: None,
                ex_oop: None,
            };

            Err(exception)
        }
    }
}

fn jvm_isPrimitive(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let v = args.get(0).unwrap();
    let v = v.lock().unwrap();
    let v = match &v.v {
        Oop::Mirror(mirror) => {
            if mirror.target.is_none() {
                1
            } else {
                0
            }
        }
        _ => unreachable!(),
    };
    Ok(Some(OopDesc::new_int(v)))
}

fn jvm_isAssignableFrom(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let l = args.get(0).unwrap();
    let r = args.get(1).unwrap();

    let (lt, ltyp) = {
        let v = l.lock().unwrap();
        match &v.v {
            Oop::Mirror(mirror) => (mirror.target.clone(), mirror.value_type),
            _ => unreachable!(),
        }
    };

    let (rt, rtyp) = {
        let v = r.lock().unwrap();
        match &v.v {
            Oop::Mirror(mirror) => (mirror.target.clone(), mirror.value_type),
            _ => unreachable!(),
        }
    };

    let v = if lt.is_none() && rt.is_none() {
        if ltyp == rtyp {
            1
        } else {
            0
        }
    } else {
        let lt = lt.unwrap();
        let rt = rt.unwrap();
        if runtime::cmp::instance_of(lt, rt) {
            1
        } else {
            0
        }
    };

    Ok(Some(OopDesc::new_int(v)))
}

fn jvm_isInterface(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let v = args.get(0).unwrap();
    let v = v.lock().unwrap();
    let v = match &v.v {
        Oop::Mirror(mirror) => match &mirror.target {
            Some(target) => {
                if target.lock().unwrap().is_interface() {
                    1
                } else {
                    0
                }
            }
            None => 0,
        },
        _ => unreachable!(),
    };
    Ok(Some(OopDesc::new_int(v)))
}

fn jvm_getDeclaredConstructors0(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    //parse args
    let mirror_target = {
        let arg0 = args.get(0).unwrap();
        let arg0 = arg0.lock().unwrap();
        match &arg0.v {
            Oop::Mirror(mirror) => mirror.target.clone().unwrap(),
            _ => unreachable!(),
        }
    };

    let public_only = {
        let arg1 = args.get(1).unwrap();
        let arg1 = arg1.lock().unwrap();
        match arg1.v {
            Oop::Int(v) => v == 1,
            _ => unreachable!(),
        }
    };

    //fixme: super methods
    let all_methods = {
        let cls = mirror_target.lock().unwrap();
        match &cls.kind {
            oop::class::ClassKind::Instance(inst) => inst.all_methods.clone(),
            _ => unreachable!(),
        }
    };

    //build methods ary
    let mut methods = Vec::new();
    for (_, m) in all_methods {
        if m.method.name.as_slice() == b"<init>" {
            let v = runtime::reflect::new_method_ctor(jt, m);
            methods.push(v);
        }
    }

    //build oop field ar
    let ary_cls = require_class3(None, b"[Ljava/lang/reflect/Constructor;").unwrap();

    Ok(Some(OopDesc::new_ary2(ary_cls, methods)))
}

pub fn jvm_getModifiers(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let v = args.get(0).unwrap();
    let v = v.lock().unwrap();
    let v = match &v.v {
        Oop::Mirror(mirror) => match &mirror.target {
            Some(target) => target.lock().unwrap().acc_flags,
            None => acc::ACC_ABSTRACT | acc::ACC_FINAL | acc::ACC_PUBLIC,
        },
        _ => unreachable!(),
    };

    Ok(Some(OopDesc::new_int(v as i32)))
}

pub fn jvm_getSuperclass(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let mirror = args.get(0).unwrap();
    let v = mirror.lock().unwrap();
    match &v.v {
        Oop::Mirror(mirror) => match &mirror.target {
            Some(target) => {
                let cls = target.lock().unwrap();
                match &cls.super_class {
                    Some(super_cls) => {
                        let cls = super_cls.lock().unwrap();
                        let mirror = cls.get_mirror();
                        Ok(Some(mirror))
                    }
                    None => Ok(Some(oop::consts::get_null())),
                }
            }
            None => Ok(Some(oop::consts::get_null())),
        },
        _ => unreachable!(),
    }
}
