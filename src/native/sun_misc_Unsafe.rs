#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{Oop, OopDesc};
use crate::runtime::{require_class3, JavaThread};
use crate::types::OopRef;
use crate::util;
use std::os::raw::c_void;

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![
        new_fn("registerNatives", "()V", Box::new(jvm_registerNatives)),
        new_fn(
            "arrayBaseOffset",
            "(Ljava/lang/Class;)I",
            Box::new(jvm_arrayBaseOffset),
        ),
        new_fn(
            "arrayIndexScale",
            "(Ljava/lang/Class;)I",
            Box::new(jvm_arrayIndexScale),
        ),
        new_fn("addressSize", "()I", Box::new(jvm_addressSize)),
        new_fn(
            "objectFieldOffset",
            "(Ljava/lang/reflect/Field;)J",
            Box::new(jvm_objectFieldOffset),
        ),
        new_fn(
            "compareAndSwapObject",
            "(Ljava/lang/Object;JLjava/lang/Object;Ljava/lang/Object;)Z",
            Box::new(jvm_compareAndSwapObject),
        ),
        new_fn(
            "getIntVolatile",
            "(Ljava/lang/Object;J)I",
            Box::new(jvm_getIntVolatile),
        ),
        new_fn(
            "compareAndSwapInt",
            "(Ljava/lang/Object;JII)Z",
            Box::new(jvm_compareAndSwapInt),
        ),
        new_fn("allocateMemory", "(J)J", Box::new(jvm_allocateMemory)),
        new_fn("freeMemory", "(J)V", Box::new(jvm_freeMemory)),
        new_fn("putLong", "(JJ)V", Box::new(jvm_putLong)),
        new_fn("getByte", "(J)B", Box::new(jvm_getByte)),
        new_fn(
            "compareAndSwapLong",
            "(Ljava/lang/Object;JJJ)Z",
            Box::new(jvm_compareAndSwapLong),
        ),
        new_fn(
            "getObjectVolatile",
            "(Ljava/lang/Object;J)Ljava/lang/Object;",
            Box::new(jvm_getObjectVolatile),
        ),
    ]
}

fn jvm_registerNatives(_jt: &mut JavaThread, _env: JNIEnv, _args: Vec<OopRef>) -> JNIResult {
    Ok(None)
}

fn jvm_arrayBaseOffset(_jt: &mut JavaThread, _env: JNIEnv, _args: Vec<OopRef>) -> JNIResult {
    Ok(Some(OopDesc::new_int(0)))
}

fn jvm_arrayIndexScale(_jt: &mut JavaThread, _env: JNIEnv, _args: Vec<OopRef>) -> JNIResult {
    //    let v = std::mem::size_of::<*mut u8>();
    //    Ok(Some(OopDesc::new_int(v as i32)))
    Ok(Some(OopDesc::new_int(1)))
}

fn jvm_addressSize(_jt: &mut JavaThread, _env: JNIEnv, _args: Vec<OopRef>) -> JNIResult {
    let v = std::mem::size_of::<*mut u8>();
    Ok(Some(OopDesc::new_int(v as i32)))
}

fn jvm_objectFieldOffset(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let field = args[1].clone();

    {
        let v = field.lock().unwrap();
        match &v.v {
            Oop::Inst(inst) => {
                let cls = inst.class.clone();
                let cls = cls.lock().unwrap();
                assert_eq!(cls.name.as_slice(), b"java/lang/reflect/Field");
            }
            _ => unreachable!(),
        }
    }

    let cls = require_class3(None, b"java/lang/reflect/Field").unwrap();
    let v = {
        let cls = cls.lock().unwrap();
        let id = cls.get_field_id(b"slot", b"I", false);
        cls.get_field_value(field, id)
    };
    let v = v.lock().unwrap();
    let v = match &v.v {
        Oop::Int(i) => OopDesc::new_long(*i as i64),
        _ => unreachable!(),
    };

    Ok(Some(v))
}

//fixme: 此处语义上要求是原子操作，这里需要重新实现
fn jvm_compareAndSwapObject(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let owner = args.get(1).unwrap();
    let offset = {
        let v = args.get(2).unwrap();
        let v = v.lock().unwrap();
        match v.v {
            Oop::Long(v) => v,
            _ => unreachable!(),
        }
    };
    let old_data = args.get(3).unwrap();
    let new_data = args.get(4).unwrap();

    let v_at_offset = {
        let v = owner.lock().unwrap();
        match &v.v {
            Oop::Mirror(mirror) => mirror.field_values[offset as usize].clone(),
            Oop::Array(ary) => ary.elements[offset as usize].clone(),
            t => unreachable!("{:?}", t),
        }
    };

    if util::oop::if_acmpeq(v_at_offset, old_data.clone()) {
        let mut v = owner.lock().unwrap();
        match &mut v.v {
            Oop::Mirror(mirror) => {
                mirror.field_values[offset as usize] = new_data.clone();
            }
            Oop::Array(ary) => {
                ary.elements[offset as usize] = new_data.clone();
            }
            _ => unreachable!(),
        }

        Ok(Some(OopDesc::new_int(1)))
    } else {
        Ok(Some(OopDesc::new_int(0)))
    }
}

fn jvm_getIntVolatile(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let owner = args.get(1).unwrap();
    let offset = {
        let v = args.get(2).unwrap();
        let v = v.lock().unwrap();
        match v.v {
            Oop::Long(v) => v,
            _ => unreachable!(),
        }
    };
    let v_at_offset = {
        let v = owner.lock().unwrap();
        match &v.v {
            Oop::Inst(inst) => inst.field_values[offset as usize].clone(),
            _ => unreachable!(),
        }
    };
    Ok(Some(v_at_offset))
}

fn jvm_compareAndSwapInt(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let owner = args.get(1).unwrap();
    let offset = {
        let v = args.get(2).unwrap();
        let v = v.lock().unwrap();
        match v.v {
            Oop::Long(v) => v,
            _ => unreachable!(),
        }
    };
    let old_data = args.get(3).unwrap();
    let old_data = {
        let v = old_data.lock().unwrap();
        match v.v {
            Oop::Int(v) => v,
            _ => unreachable!(),
        }
    };
    let new_data = args.get(4).unwrap();

    let v_at_offset = {
        let v = owner.lock().unwrap();
        let v = match &v.v {
            Oop::Inst(inst) => inst.field_values[offset as usize].clone(),
            _ => unreachable!(),
        };
        let v = v.lock().unwrap();
        match v.v {
            Oop::Int(v) => v,
            _ => unreachable!(),
        }
    };

    if v_at_offset == old_data {
        let mut v = owner.lock().unwrap();
        match &mut v.v {
            Oop::Inst(inst) => inst.field_values[offset as usize] = new_data.clone(),
            _ => unreachable!(),
        }

        Ok(Some(OopDesc::new_int(1)))
    } else {
        Ok(Some(OopDesc::new_int(0)))
    }
}

fn jvm_allocateMemory(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let size = {
        let v = args.get(1).unwrap();
        let v = v.lock().unwrap();
        match v.v {
            Oop::Long(v) => v as usize,
            _ => unreachable!(),
        }
    };

    let arr = unsafe { libc::malloc(std::mem::size_of::<u8>() * size) };
    let v = arr as i64;

    Ok(Some(OopDesc::new_long(v)))
}

fn jvm_freeMemory(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let ptr = {
        let v = args.get(1).unwrap();
        let v = v.lock().unwrap();
        match v.v {
            Oop::Long(v) => v as *mut c_void,
            _ => unreachable!(),
        }
    };

    unsafe {
        libc::free(ptr);
    }

    Ok(None)
}

fn jvm_putLong(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let ptr = {
        let v = args.get(1).unwrap();
        let v = v.lock().unwrap();
        match v.v {
            Oop::Long(v) => v as *mut c_void,
            _ => unreachable!(),
        }
    };
    let l = {
        let v = args.get(2).unwrap();
        let v = v.lock().unwrap();
        match v.v {
            Oop::Long(v) => v,
            _ => unreachable!(),
        }
    };
    let v = l.to_be_bytes();
    let v = vec![v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]];
    unsafe {
        libc::memcpy(ptr, v.as_ptr() as *const c_void, 8);
    }

    Ok(None)
}

fn jvm_getByte(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let ptr = {
        let v = args.get(1).unwrap();
        let v = v.lock().unwrap();
        match v.v {
            Oop::Long(v) => v as *mut u8,
            _ => unreachable!(),
        }
    };
    let v = unsafe { *ptr };
    Ok(Some(OopDesc::new_int(v as i32)))
}

fn jvm_compareAndSwapLong(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let owner = args.get(1).unwrap();
    let offset = {
        let v = args.get(2).unwrap();
        let v = v.lock().unwrap();
        match v.v {
            Oop::Long(v) => v,
            _ => unreachable!(),
        }
    };
    let old_data = args.get(3).unwrap();
    let old_data = {
        let v = old_data.lock().unwrap();
        match v.v {
            Oop::Long(v) => v,
            _ => unreachable!(),
        }
    };
    let new_data = args.get(4).unwrap();

    let v_at_offset = {
        let v = owner.lock().unwrap();
        let v = match &v.v {
            Oop::Inst(inst) => inst.field_values[offset as usize].clone(),
            _ => unreachable!(),
        };
        let v = v.lock().unwrap();
        match v.v {
            Oop::Long(v) => v,
            _ => unreachable!(),
        }
    };

    if v_at_offset == old_data {
        let mut v = owner.lock().unwrap();
        match &mut v.v {
            Oop::Inst(inst) => inst.field_values[offset as usize] = new_data.clone(),
            _ => unreachable!(),
        }

        Ok(Some(OopDesc::new_int(1)))
    } else {
        Ok(Some(OopDesc::new_int(0)))
    }
}

fn jvm_getObjectVolatile(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let owner = args.get(1).unwrap();
    let offset = {
        let v = args.get(2).unwrap();
        let v = v.lock().unwrap();
        match v.v {
            Oop::Long(v) => v,
            _ => unreachable!(),
        }
    };
    let v_at_offset = {
        let v = owner.lock().unwrap();
        match &v.v {
            Oop::Inst(inst) => inst.field_values[offset as usize].clone(),
            Oop::Array(ary) => ary.elements[offset as usize].clone(),
            t => unreachable!("t = {:?}", t),
        }
    };
    Ok(Some(v_at_offset))
}
