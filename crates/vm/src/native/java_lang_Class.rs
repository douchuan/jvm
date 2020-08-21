#![allow(non_snake_case)]

use crate::native::{common, new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{self, Class, ClassKind, Oop, OopRef, ValueType};
use crate::runtime::{self, require_class2, require_class3};
use crate::types::{ClassRef, MethodIdRef};
use crate::util;
use classfile::{constant_pool, consts as cls_consts, flags as acc};
use rustc_hash::FxHashMap;
use std::sync::{Arc, RwLock};

pub fn get_primitive_class_mirror(key: &str) -> Option<Oop> {
    //todo: avoid mutex lock, it's only read
    let mirrors = PRIM_MIRROS.read().unwrap();
    mirrors.get(key).cloned()
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
        new_fn("isArray", "()Z", Box::new(jvm_isArray)),
        new_fn(
            "getComponentType",
            "()Ljava/lang/Class;",
            Box::new(jvm_getComponentType),
        ),
        new_fn(
            "getEnclosingMethod0",
            "()[Ljava/lang/Object;",
            Box::new(jvm_getEnclosingMethod0),
        ),
        new_fn(
            "getDeclaringClass0",
            "()Ljava/lang/Class;",
            Box::new(jvm_getDeclaringClass0),
        ),
        new_fn(
            "isInstance",
            "(Ljava/lang/Object;)Z",
            Box::new(jvm_isInstance),
        ),
        new_fn(
            "getDeclaredMethods0",
            "(Z)[Ljava/lang/reflect/Method;",
            Box::new(jvm_getDeclaredMethods0),
        ),
        new_fn(
            "getInterfaces0",
            "()[Ljava/lang/Class;",
            Box::new(jvm_getInterfaces0),
        ),
        new_fn("getRawAnnotations", "()[B", Box::new(jvm_getRawAnnotations)),
        new_fn(
            "getConstantPool",
            "()Lsun/reflect/ConstantPool;",
            Box::new(jvm_getConstantPool),
        ),
        new_fn(
            "getDeclaredClasses0",
            "()[Ljava/lang/Class;",
            Box::new(jvm_getDeclaredClasses0),
        ),
        new_fn(
            "getGenericSignature0",
            "()Ljava/lang/String;",
            Box::new(jvm_getGenericSignature0),
        ),
    ]
}

#[derive(Copy, Clone, PartialEq)]
enum ClassMirrorState {
    NotFixed,
    Fixed,
}

lazy_static! {
    static ref MIRROR_STATE: RwLock<ClassMirrorState> = RwLock::new(ClassMirrorState::NotFixed);
    static ref PRIM_MIRROS: RwLock<FxHashMap<String, Oop>> = {
        let hm = FxHashMap::default();
        RwLock::new(hm)
    };
    static ref SIGNATURE_DIC: FxHashMap<&'static str, &'static str> = {
        let dic: FxHashMap<&'static str, &'static str> = [
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
    static ref DELAYED_MIRROS: RwLock<Vec<String>> = {
        let v = vec![
            "I", "Z", "B", "C", "S", "F", "J", "D", "V", "[I", "[Z", "[B", "[C", "[S", "[F", "[J",
            "[D",
        ];
        let v: Vec<String> = v.iter().map(|it| it.to_string()).collect();
        RwLock::new(v)
    };
    static ref DELAYED_ARY_MIRROS: RwLock<Vec<ClassRef>> = {
        let v = vec![];
        RwLock::new(v)
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
    let is_fixed = {
        let s = MIRROR_STATE.write().unwrap();
        *s == ClassMirrorState::Fixed
    };

    if is_fixed {
        let mirror = Oop::new_mirror(cls.clone());
        let cls = cls.get_mut_class();
        trace!("mirror created: {}", unsafe {
            std::str::from_utf8_unchecked(cls.name.as_slice())
        });
        cls.set_mirror(mirror);
    } else {
        let cls_back = cls.clone();
        let cls = cls.get_class();
        let name = unsafe { std::str::from_utf8_unchecked(cls.name.as_slice()) };
        warn!("mirror create delayed: {}", name);
        match cls.kind {
            oop::class::ClassKind::Instance(_) => {
                let mut mirrors = DELAYED_MIRROS.write().unwrap();
                mirrors.push(String::from(name));
            }
            _ => {
                let mut mirrors = DELAYED_ARY_MIRROS.write().unwrap();
                mirrors.push(cls_back);
            }
        }
    }
}

/*
called after 'java/lang/Class' inited in init_vm.rs
*/
pub fn create_delayed_mirrors() {
    let names: Vec<String> = {
        let mirros = DELAYED_MIRROS.read().unwrap();
        mirros.clone()
    };

    {
        let mut s = MIRROR_STATE.write().unwrap();
        *s = ClassMirrorState::Fixed;
    }

    for name in names {
        if name.len() > 2 {
            //java.lang.XXX
            let target = require_class3(None, name.as_bytes()).unwrap();
            create_mirror(target);
        } else {
            let is_prim_ary = name.as_bytes()[0] == b'[';
            let (vt, target) = if is_prim_ary {
                let vt = ValueType::from(&name.as_bytes()[1]);
                let target = require_class3(None, name.as_bytes()).unwrap();
                (vt, Some(target))
            } else {
                (ValueType::from(&name.as_bytes()[0]), None)
            };

            let mirror = Oop::new_prim_mirror(vt, target.clone());
            if is_prim_ary {
                let target = target.unwrap();
                let cls = target.get_mut_class();
                //                warn!("set_mirror name={}", String::from_utf8_lossy(cls.name.as_slice()));
                cls.set_mirror(mirror.clone());
            }

            let mut mirrors = PRIM_MIRROS.write().unwrap();
            mirrors.insert(name.to_string(), mirror);
        }
    }
}

/*
called after 'java/lang/Class' inited in init_vm.rs
*/
pub fn create_delayed_ary_mirrors() {
    let classes: Vec<ClassRef> = {
        let mirros = DELAYED_ARY_MIRROS.read().unwrap();
        mirros.clone()
    };

    for cls in classes {
        let value_type = {
            let cls = cls.get_class();
            match &cls.kind {
                oop::class::ClassKind::ObjectArray(obj_ary) => obj_ary.value_type,
                oop::class::ClassKind::TypeArray(typ_ary) => typ_ary.value_type,
                _ => unreachable!(),
            }
        };
        let mirror = Oop::new_ary_mirror(cls.clone(), value_type);
        let cls = cls.get_mut_class();
        cls.set_mirror(mirror);
    }
}

fn jvm_registerNatives(_env: JNIEnv, _args: &Vec<Oop>) -> JNIResult {
    Ok(None)
}

fn jvm_desiredAssertionStatus0(_env: JNIEnv, _args: &Vec<Oop>) -> JNIResult {
    Ok(Some(Oop::new_int(0)))
}

fn jvm_getPrimitiveClass(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let v = args.get(0).unwrap();
    let v = OopRef::java_lang_string(v.extract_ref());
    match SIGNATURE_DIC.get(v.as_str()) {
        Some(&s) => Ok(get_primitive_class_mirror(s)),
        _ => unreachable!("Unknown primitive type: {}", v),
    }
}

fn jvm_getDeclaredFields0(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    //parse args
    let mirror_target = {
        let arg0 = args.get(0).unwrap();
        extract_mirror_target(arg0)
    };

    let public_only = {
        let arg1 = args.get(1).unwrap();
        arg1.extract_int() == 1
    };

    //fixme: super fields
    //obtain inst&static fields
    let (inst_fields, static_fields) = {
        let inst = mirror_target.extract_inst();
        (inst.inst_fields.clone(), inst.static_fields.clone())
    };

    //build fields ary
    let mut fields = Vec::new();
    for (_, it) in inst_fields {
        if public_only && !it.field.is_public() {
            continue;
        }

        let v = common::reflect::new_field(it);
        fields.push(v);
    }

    for (_, it) in static_fields {
        if public_only && !it.field.is_public() {
            continue;
        }

        let v = common::reflect::new_field(it);
        fields.push(v);
    }

    //build oop field ar
    let ary_cls = require_class3(None, b"[Ljava/lang/reflect/Field;").unwrap();
    Ok(Some(Oop::new_ref_ary2(ary_cls, fields)))
}

fn jvm_getName0(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let (target, vt) = {
        let arg0 = args.get(0).unwrap();
        let rf = arg0.extract_ref();
        let mirror = rf.extract_mirror();
        (mirror.target.clone(), mirror.value_type)
    };
    let name = {
        match target {
            Some(target) => {
                let cls = target.get_class();
                Vec::from(cls.name.as_slice())
            }
            None => {
                let v = vt.get_primitive_name();
                Vec::from(v)
            }
        }
    };

    let name = unsafe { std::str::from_utf8_unchecked(name.as_slice()) };
    let name = name.replace("/", ".");
    let v = util::oop::new_java_lang_string2(&name);
    Ok(Some(v))
}

fn jvm_forName0(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let arg0 = args.get(0).unwrap();
    let java_name = {
        let rf = arg0.extract_ref();
        OopRef::java_lang_string(rf)
    };
    let initialize = {
        let arg1 = args.get(1).unwrap();
        arg1.extract_int() != 0
    };
    let java_cls_loader = args.get(2).unwrap();
    {
        match java_cls_loader {
            Oop::Null => (),
            _ => unimplemented!("app class loader, unimpl"),
        }
    }

    let _caller_mirror = args.get(3).unwrap();

    if java_name.contains('/') {
        let msg = Some(java_name);
        let ex = runtime::exception::new(cls_consts::J_CLASS_NOT_FOUND, msg);
        return Err(ex);
    }

    let java_name = java_name.replace(".", "/");
    let cls = {
        if java_name == "sun/nio/cs/ext/ExtendedCharsets" {
            //fixme: skip, cause jvm start very slow
            None
        } else {
            require_class3(None, java_name.as_bytes())
        }
    };

    match cls {
        Some(cls) => {
            oop::class::init_class(&cls);
            if initialize {
                oop::class::init_class_fully(&cls);
            }

            let mirror = cls.get_class().get_mirror();
            Ok(Some(mirror))
        }
        None => {
            // error!("forName0, NotFound: {}", java_name);
            let msg = Some(java_name);
            let ex = runtime::exception::new(cls_consts::J_CLASS_NOT_FOUND, msg);
            Err(ex)
        }
    }
}

fn jvm_isPrimitive(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let v = args.get(0).unwrap();
    let v = {
        let rf = v.extract_ref();
        let mirror = rf.extract_mirror();
        if mirror.target.is_none() {
            1
        } else {
            0
        }
    };
    Ok(Some(Oop::new_int(v)))
}

fn jvm_isAssignableFrom(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let l = args.get(0).unwrap();
    let r = args.get(1).unwrap();

    let (lt, ltyp) = {
        let rf = l.extract_ref();
        let mirror = rf.extract_mirror();
        (mirror.target.clone(), mirror.value_type)
    };

    let (rt, rtyp) = {
        let rf = r.extract_ref();
        let mirror = rf.extract_mirror();
        (mirror.target.clone(), mirror.value_type)
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
        if runtime::cmp::instance_of(rt, lt) {
            1
        } else {
            0
        }
    };

    Ok(Some(Oop::new_int(v)))
}

fn jvm_isInterface(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let v = args.get(0).unwrap();
    let v = {
        let rf = v.extract_ref();
        let mirror = rf.extract_mirror();
        match &mirror.target {
            Some(target) => {
                if target.get_class().is_interface() {
                    1
                } else {
                    0
                }
            }
            None => 0,
        }
    };

    Ok(Some(Oop::new_int(v)))
}

fn jvm_getDeclaredConstructors0(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    //parse args
    let mirror_target = {
        let arg0 = args.get(0).unwrap();
        extract_mirror_target(arg0)
    };
    let public_only = args.get(1).unwrap().extract_int() == 1;
    get_declared_method_helper(mirror_target, public_only, true)
}

pub fn jvm_getModifiers(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let v = args.get(0).unwrap();
    let v = {
        let rf = v.extract_ref();
        let mirror = rf.extract_mirror();
        match &mirror.target {
            Some(target) => {
                let mut acc_flags = target.get_class().acc_flags;

                //use access_flags in InnerClasses Attribute
                //don't know why. just read the JDK codes
                let inst = target.extract_inst();
                let this_class = inst.class_file.this_class;
                match &inst.inner_classes {
                    Some(inner_classes) => {
                        for it in inner_classes {
                            if it.inner_class_info_index == 0 {
                                continue;
                            }

                            if it.inner_class_info_index == this_class {
                                acc_flags = it.inner_class_access_flags;
                            }
                        }
                    }
                    None => {}
                }

                acc_flags
            }
            None => acc::ACC_ABSTRACT | acc::ACC_FINAL | acc::ACC_PUBLIC,
        }
    };

    Ok(Some(Oop::new_int(v as i32)))
}

fn jvm_getSuperclass(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let arg0 = args.get(0).unwrap();
    let rf = arg0.extract_ref();
    let mirror = rf.extract_mirror();
    match &mirror.target {
        Some(target) => {
            let cls = target.get_class();
            match &cls.super_class {
                Some(super_cls) => {
                    let cls = super_cls.get_class();
                    let mirror = cls.get_mirror();
                    Ok(Some(mirror))
                }
                None => Ok(Some(oop::consts::get_null())),
            }
        }
        None => Ok(Some(oop::consts::get_null())),
    }
}

fn jvm_isArray(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let arg0 = args.get(0).unwrap();

    let mirror_cls = {
        let rf = arg0.extract_ref();
        let mirror = rf.extract_mirror();
        match &mirror.target {
            Some(target) => target.clone(),
            None => return Ok(Some(Oop::new_int(0))),
        }
    };

    let cls = mirror_cls.get_class();
    let v = match cls.get_class_kind_type() {
        oop::class::ClassKindType::Instance => 0,
        _ => 1,
    };

    Ok(Some(Oop::new_int(v)))
}

fn jvm_getComponentType(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let arg0 = args.get(0).unwrap();
    let cls = {
        let rf = arg0.extract_ref();
        let mirror = rf.extract_mirror();
        mirror.target.clone().unwrap()
    };
    let cls = cls.get_class();
    let v = match &cls.kind {
        oop::class::ClassKind::TypeArray(type_ary_cls) => {
            let vt = type_ary_cls.value_type.into();
            let key = unsafe { std::str::from_utf8_unchecked(vt) };
            let mirrors = PRIM_MIRROS.read().unwrap();
            mirrors.get(key).cloned()
        }
        oop::class::ClassKind::ObjectArray(obj_ary_cls) => {
            let component = obj_ary_cls.component.clone().unwrap();
            let cls = component.get_class();
            Some(cls.get_mirror())
        }
        _ => unreachable!(),
    };
    Ok(v)
}

fn jvm_getEnclosingMethod0(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let arg0 = args.get(0).unwrap();
    let target = {
        let rf = arg0.extract_ref();
        let ptr = rf.get_raw_ptr();
        unsafe {
            match &(*ptr).v {
                oop::RefKind::Mirror(mirror) => mirror.target.clone(),
                _ => return Ok(Some(oop::consts::get_null())),
            }
        }
    };

    let (cls_file, em) = match target {
        Some(target) => {
            let cls = target.get_class();
            match &cls.kind {
                ClassKind::Instance(cls) => match &cls.enclosing_method {
                    Some(em) => (cls.class_file.clone(), em.clone()),
                    None => return Ok(Some(oop::consts::get_null())),
                },
                _ => return Ok(Some(oop::consts::get_null())),
            }
        }
        None => return Ok(Some(oop::consts::get_null())),
    };

    //push EnclosingMethod class mirror
    if em.class_index == 0 {
        panic!();
    }
    let em_class = require_class2(em.class_index, &cls_file.cp).unwrap();
    let em_class_mirror = {
        let cls = em_class.get_class();
        cls.get_mirror()
    };
    let mut elms = Vec::with_capacity(3);
    elms.push(em_class_mirror);

    //push EnclosingMethod name&desc
    if em.method_index != 0 {
        let (name, desc) = constant_pool::get_name_and_type(&cls_file.cp, em.method_index as usize);
        let name = unsafe { std::str::from_utf8_unchecked(name.as_slice()) };
        let desc = unsafe { std::str::from_utf8_unchecked(desc.as_slice()) };

        elms.push(util::oop::new_java_lang_string2(name));
        elms.push(util::oop::new_java_lang_string2(desc));
    } else {
        elms.push(oop::consts::get_null());
        elms.push(oop::consts::get_null());
    }

    let ary = require_class3(None, b"[Ljava/lang/Object;").unwrap();
    let ary = Oop::new_ref_ary2(ary, elms);

    Ok(Some(ary))
}

fn jvm_getDeclaringClass0(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let arg0 = args.get(0).unwrap();
    let target = {
        let rf = arg0.extract_ref();
        let ptr = rf.get_raw_ptr();
        unsafe {
            match &(*ptr).v {
                oop::RefKind::Mirror(mirror) => mirror.target.clone(),
                _ => return Ok(Some(oop::consts::get_null())),
            }
        }
    };

    let (cls_file, target, inner_classes) = match target {
        Some(target) => {
            let cls = target.get_class();
            match &cls.kind {
                ClassKind::Instance(cls) => match &cls.inner_classes {
                    Some(inner_classes) => (
                        cls.class_file.clone(),
                        target.clone(),
                        inner_classes.clone(),
                    ),
                    None => return Ok(Some(oop::consts::get_null())),
                },
                _ => return Ok(Some(oop::consts::get_null())),
            }
        }
        None => return Ok(Some(oop::consts::get_null())),
    };

    for it in inner_classes.iter() {
        if it.inner_class_info_index == 0 {
            continue;
        }

        let inner_class = require_class2(it.inner_class_info_index, &cls_file.cp).unwrap();

        if Arc::ptr_eq(&inner_class, &target) {
            return if it.outer_class_info_index == 0 {
                Ok(Some(oop::consts::get_null()))
            } else {
                let outer_class = require_class2(it.outer_class_info_index, &cls_file.cp).unwrap();
                let v = outer_class.get_class();
                Ok(Some(v.get_mirror()))
            };
        }
    }

    Ok(Some(oop::consts::get_null()))
}

fn jvm_isInstance(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let arg0 = args.get(0).unwrap();
    let arg1 = args.get(1).unwrap();

    let target_cls = {
        let rf = arg0.extract_ref();
        let ptr = rf.get_raw_ptr();
        unsafe {
            match &(*ptr).v {
                oop::RefKind::Inst(inst) => inst.class.clone(),
                oop::RefKind::Mirror(mirror) => mirror.target.clone().unwrap(),
                _ => unreachable!(),
            }
        }
    };
    let obj_cls = {
        let rf = arg1.extract_ref();
        let inst = rf.extract_inst();
        inst.class.clone()
    };

    let v = if runtime::cmp::instance_of(obj_cls, target_cls) {
        1
    } else {
        0
    };

    Ok(Some(Oop::new_int(v)))
}

fn jvm_getDeclaredMethods0(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    //parse args
    let mirror_target = {
        let arg0 = args.get(0).unwrap();
        extract_mirror_target(arg0)
    };
    let public_only = args.get(1).unwrap().extract_int() == 1;
    get_declared_method_helper(mirror_target, public_only, false)
}

fn jvm_getInterfaces0(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let mirror = {
        let arg0 = args.get(0).unwrap();
        extract_mirror_target(arg0)
    };

    let v = mirror.get_class();
    let elms = match &v.kind {
        oop::ClassKind::Instance(inst) => {
            let mut elms = Vec::with_capacity(inst.class_file.interfaces.len());
            let cp = &inst.class_file.cp;
            inst.class_file.interfaces.iter().for_each(|it| {
                let cls = require_class2(*it, cp).unwrap();
                let cls = cls.get_class();
                elms.push(cls.get_mirror());
            });

            elms
        }
        ClassKind::ObjectArray(_ary) => {
            let cls_cloneable = require_class3(None, cls_consts::J_CLONEABLE).unwrap();
            let cls_serializable = require_class3(None, cls_consts::J_SERIALIZABLE).unwrap();
            let mut elms = Vec::with_capacity(2);

            {
                let cls = cls_cloneable.get_class();
                elms.push(cls.get_mirror());
            }

            {
                let cls = cls_serializable.get_class();
                elms.push(cls.get_mirror());
            }

            elms
        }
        ClassKind::TypeArray(_) => unimplemented!("type array getInterfaces0"),
    };

    let clazz = require_class3(None, b"[Ljava/lang/Class;").unwrap();
    let ary = Oop::new_ref_ary2(clazz, elms);

    Ok(Some(ary))
}

fn jvm_getRawAnnotations(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let cls = args.get(0).unwrap();
    let annotations = match cls {
        Oop::Ref(rf) => {
            let mirror = rf.extract_mirror();
            let cls = mirror.target.clone().unwrap();
            let cls = cls.get_class();
            let raw = cls.get_annotation();
            match raw {
                Some(raw) => Oop::new_byte_ary2(raw.to_vec()),
                None => oop::consts::get_null(),
            }
        }
        _ => oop::consts::get_null(),
    };

    Ok(Some(annotations))
}

fn jvm_getConstantPool(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let this = args.get(0).unwrap();
    let cp_oop = match this {
        Oop::Ref(_rf) => {
            let cp_cls = require_class3(None, b"sun/reflect/ConstantPool").unwrap();
            let cp_oop = Oop::new_inst(cp_cls.clone());

            let cls = cp_cls.get_class();
            let fid =
                cls.get_field_id(&util::S_CONSTANT_POOL_OOP, &util::S_JAVA_LANG_OBJECT, false);
            //todo: reimpl maybe, create one JNIHandles, like jdk
            Class::put_field_value(cp_oop.extract_ref(), fid, this.clone());

            cp_oop
        }
        _ => oop::consts::get_null(),
    };

    Ok(Some(cp_oop))
}

fn jvm_getDeclaredClasses0(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let this = args.get(0).unwrap();
    let target_clz = extract_mirror_target(this);

    let array_class = require_class3(None, b"[Ljava/lang/Class;").unwrap();

    let target_class = target_clz.get_class();
    let v = match &target_class.kind {
        ClassKind::Instance(inst) => {
            let cp = &inst.class_file.cp;
            match &inst.inner_classes {
                Some(inner_classes) => {
                    let this_class = inst.class_file.this_class;
                    let mut inners = Vec::with_capacity(inner_classes.len());
                    for it in inner_classes {
                        if it.outer_class_info_index == this_class {
                            let inner_clz = require_class2(it.inner_class_info_index, cp).unwrap();
                            let v = inner_clz.get_class().get_mirror();
                            inners.push(v);
                        }
                    }

                    Oop::new_ref_ary2(array_class, inners)
                }
                _ => Oop::new_ref_ary(array_class, 0),
            }
        }
        _ => Oop::new_ref_ary(array_class, 0),
    };

    Ok(Some(v))
}

fn jvm_getGenericSignature0(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let this = args.get(0).unwrap();
    let v = match this {
        Oop::Ref(rf) => {
            let ptr = rf.get_raw_ptr();
            unsafe {
                let mirror = (*ptr).v.extract_mirror();
                let vt = mirror.value_type;
                if vt == ValueType::OBJECT {
                    let target = mirror.target.clone().unwrap();
                    let cls = target.get_class();
                    let sig = cls.get_attr_signatrue();
                    sig.map_or_else(oop::consts::get_null, |v| {
                        let sig = std::str::from_utf8_unchecked(v.as_slice());
                        util::oop::new_java_lang_string2(sig)
                    })
                } else {
                    oop::consts::get_null()
                }
            }
        }
        _ => unreachable!(),
    };

    Ok(Some(v))
}

fn get_declared_method_helper(
    mirror_target: ClassRef,
    public_only: bool,
    want_constructor: bool,
) -> JNIResult {
    //fixme: super methods
    let selected_methods = {
        let cls = mirror_target.get_class();
        match &cls.kind {
            oop::class::ClassKind::Instance(inst) => {
                fn chooser1(want_constructor: bool, name: &[u8]) -> bool {
                    if want_constructor {
                        name == b"<init>"
                    } else {
                        name != b"<init>"
                    }
                }

                fn chooser2(want_constructor: bool, m: &MethodIdRef) -> bool {
                    if want_constructor {
                        m.method.name.as_slice() == b"<init>" && !m.method.is_static()
                    } else {
                        m.method.name.as_slice() != b"<init>"
                    }
                }

                let mut selected_methods = Vec::new();
                for (k, m) in inst.all_methods.iter() {
                    if !chooser1(want_constructor, k.0.as_slice()) {
                        continue;
                    }

                    if chooser2(want_constructor, &m) && (!public_only || m.method.is_public()) {
                        selected_methods.push(m.clone());
                    }
                }

                selected_methods
            }
            oop::class::ClassKind::ObjectArray(_ary) => vec![],
            _ => unreachable!("{:?}", mirror_target),
        }
    };

    //build methods ary
    let mut methods = Vec::with_capacity(selected_methods.len());
    for m in selected_methods {
        let v = if want_constructor {
            common::reflect::new_method_ctor(m)
        } else {
            common::reflect::new_method_normal(m)
        };

        methods.push(v);
    }

    //build oop methods ary
    let ary_cls = if want_constructor {
        require_class3(None, b"[Ljava/lang/reflect/Constructor;").unwrap()
    } else {
        require_class3(None, b"[Ljava/lang/reflect/Method;").unwrap()
    };

    Ok(Some(Oop::new_ref_ary2(ary_cls, methods)))
}

fn extract_mirror_target(v: &Oop) -> ClassRef {
    let rf = v.extract_ref();
    let mirror = rf.extract_mirror();
    mirror.target.clone().unwrap()
}
