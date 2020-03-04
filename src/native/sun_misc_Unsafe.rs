#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{self, Oop};
use crate::runtime::{require_class3, JavaThread};
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
        new_fn("pageSize", "()I", Box::new(jvm_pageSize)),
        new_fn(
            "getLongVolatile",
            "(Ljava/lang/Object;J)J",
            Box::new(jvm_getLongVolatile),
        ),
        new_fn(
            "setMemory",
            "(Ljava/lang/Object;JJB)V",
            Box::new(jvm_setMemory),
        ),
        new_fn("putChar", "(JC)V", Box::new(jvm_putChar)),
        new_fn(
            "copyMemory",
            "(Ljava/lang/Object;JLjava/lang/Object;JJ)V",
            Box::new(jvm_copyMemory),
        ),
        new_fn("getChar", "(J)C", Box::new(jvm_getChar)),
    ]
}

fn jvm_registerNatives(_jt: &mut JavaThread, _env: JNIEnv, _args: Vec<Oop>) -> JNIResult {
    Ok(None)
}

fn jvm_arrayBaseOffset(_jt: &mut JavaThread, _env: JNIEnv, _args: Vec<Oop>) -> JNIResult {
    Ok(Some(Oop::new_int(0)))
}

fn jvm_arrayIndexScale(_jt: &mut JavaThread, _env: JNIEnv, _args: Vec<Oop>) -> JNIResult {
    //    let v = std::mem::size_of::<*mut u8>();
    //    Ok(Some(OopDesc::new_int(v as i32)))
    Ok(Some(Oop::new_int(1)))
}

fn jvm_addressSize(_jt: &mut JavaThread, _env: JNIEnv, _args: Vec<Oop>) -> JNIResult {
    let v = std::mem::size_of::<*mut u8>();
    Ok(Some(Oop::new_int(v as i32)))
}

fn jvm_objectFieldOffset(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    let field = args.get(1).unwrap();

    {
        let field = util::oop::extract_ref(field);
        let v = field.read().unwrap();
        match &v.v {
            oop::RefKind::Inst(inst) => {
                let cls = inst.class.clone();
                let cls = cls.read().unwrap();
                assert_eq!(cls.name.as_slice(), b"java/lang/reflect/Field");
            }
            _ => unreachable!(),
        }
    }

    let cls = require_class3(None, b"java/lang/reflect/Field").unwrap();
    let v = {
        let cls = cls.read().unwrap();
        let id = cls.get_field_id(b"slot", b"I", false);
        cls.get_field_value(field, id)
    };

    let v = util::oop::extract_int(&v);

    Ok(Some(Oop::new_long(v as i64)))
}

// fixme: The semantic requirement here is atomic operation, which needs to be re-implemented here
fn jvm_compareAndSwapObject(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    let owner = args.get(1).unwrap();
    let offset = util::oop::extract_long(args.get(2).unwrap());
    let old_data = args.get(3).unwrap();
    let new_data = args.get(4).unwrap();

    let v_at_offset = {
        let owner = util::oop::extract_ref(owner);
        let v = owner.read().unwrap();
        match &v.v {
            oop::RefKind::Mirror(mirror) => mirror.field_values[offset as usize].clone(),
            oop::RefKind::Array(ary) => ary.elements[offset as usize].clone(),
            oop::RefKind::Inst(inst) => inst.field_values[offset as usize].clone(),
            t => unreachable!("{:?}", t),
        }
    };

    if util::oop::if_acmpeq(&v_at_offset, old_data) {
        let owner = util::oop::extract_ref(owner);
        let mut v = owner.write().unwrap();
        match &mut v.v {
            oop::RefKind::Mirror(mirror) => {
                mirror.field_values[offset as usize] = new_data.clone();
            }
            oop::RefKind::Array(ary) => {
                ary.elements[offset as usize] = new_data.clone();
            }
            oop::RefKind::Inst(inst) => inst.field_values[offset as usize] = new_data.clone(),
            _ => unreachable!(),
        }

        Ok(Some(Oop::new_int(1)))
    } else {
        Ok(Some(Oop::new_int(0)))
    }
}

fn jvm_getIntVolatile(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    let owner = args.get(1).unwrap();
    let offset = util::oop::extract_long(args.get(2).unwrap());
    let v_at_offset = {
        let owner = util::oop::extract_ref(owner);
        let v = owner.read().unwrap();
        match &v.v {
            oop::RefKind::Inst(inst) => inst.field_values[offset as usize].clone(),
            _ => unreachable!(),
        }
    };
    Ok(Some(v_at_offset))
}

fn jvm_compareAndSwapInt(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    let owner = args.get(1).unwrap();
    let offset = util::oop::extract_long(args.get(2).unwrap());
    let old_data = util::oop::extract_int(args.get(3).unwrap());
    let new_data = args.get(4).unwrap();

    let v_at_offset = {
        let owner = util::oop::extract_ref(owner);
        let v = owner.read().unwrap();
        let v = match &v.v {
            oop::RefKind::Inst(inst) => inst.field_values[offset as usize].clone(),
            _ => unreachable!(),
        };

        util::oop::extract_int(&v)
    };

    if v_at_offset == old_data {
        let owner = util::oop::extract_ref(owner);
        let mut v = owner.write().unwrap();
        match &mut v.v {
            oop::RefKind::Inst(inst) => inst.field_values[offset as usize] = new_data.clone(),
            _ => unreachable!(),
        }

        Ok(Some(Oop::new_int(1)))
    } else {
        Ok(Some(Oop::new_int(0)))
    }
}

fn jvm_allocateMemory(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    let size = util::oop::extract_long(args.get(1).unwrap()) as usize;
    let arr = unsafe { libc::malloc(std::mem::size_of::<u8>() * size) };
    let v = arr as i64;

    Ok(Some(Oop::new_long(v)))
}

fn jvm_freeMemory(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    let ptr = util::oop::extract_long(args.get(1).unwrap()) as *mut libc::c_void;

    unsafe {
        libc::free(ptr);
    }

    Ok(None)
}

fn jvm_putLong(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    let ptr = util::oop::extract_long(args.get(1).unwrap()) as *mut libc::c_void;
    let l = util::oop::extract_long(args.get(2).unwrap());
    let v = l.to_be_bytes();
    let v = vec![v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]];
    unsafe {
        libc::memcpy(ptr, v.as_ptr() as *const c_void, 8);
    }

    Ok(None)
}

fn jvm_getByte(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    let ptr = util::oop::extract_long(args.get(1).unwrap()) as *const u8;
    let v = unsafe { *ptr };
    Ok(Some(Oop::new_int(v as i32)))
}

fn jvm_compareAndSwapLong(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    let owner = args.get(1).unwrap();
    let offset = util::oop::extract_long(args.get(2).unwrap());
    let old_data = util::oop::extract_long(args.get(3).unwrap());
    let new_data = args.get(4).unwrap();

    let v_at_offset = {
        let owner = util::oop::extract_ref(owner);
        let v = owner.read().unwrap();
        let v = match &v.v {
            oop::RefKind::Inst(inst) => inst.field_values[offset as usize].clone(),
            _ => unreachable!(),
        };

        util::oop::extract_long(&v)
    };

    if v_at_offset == old_data {
        let owner = util::oop::extract_ref(owner);
        let mut v = owner.write().unwrap();
        match &mut v.v {
            oop::RefKind::Inst(inst) => inst.field_values[offset as usize] = new_data.clone(),
            _ => unreachable!(),
        }

        Ok(Some(Oop::new_int(1)))
    } else {
        Ok(Some(Oop::new_int(0)))
    }
}

fn jvm_getObjectVolatile(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    let owner = args.get(1).unwrap();
    let offset = util::oop::extract_long(args.get(2).unwrap());
    let v_at_offset = {
        let owner = util::oop::extract_ref(owner);
        let v = owner.read().unwrap();
        match &v.v {
            oop::RefKind::Inst(inst) => inst.field_values[offset as usize].clone(),
            oop::RefKind::Array(ary) => ary.elements[offset as usize].clone(),
            t => unreachable!("t = {:?}", t),
        }
    };
    Ok(Some(v_at_offset))
}

fn jvm_pageSize(_jt: &mut JavaThread, _env: JNIEnv, _args: Vec<Oop>) -> JNIResult {
    Ok(Some(Oop::new_int(4 * 1024)))
}

fn jvm_getLongVolatile(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    let owner = args.get(1).unwrap();
    let offset = util::oop::extract_long(args.get(2).unwrap());
    let v_at_offset = {
        let owner = util::oop::extract_ref(owner);
        let v = owner.read().unwrap();
        match &v.v {
            oop::RefKind::Inst(inst) => inst.field_values[offset as usize].clone(),
            _ => unreachable!(),
        }
    };
    Ok(Some(v_at_offset))
}

fn jvm_setMemory(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    let _this = args.get(0).unwrap();
    let obj = args.get(1).unwrap();
    let offset = util::oop::extract_long(args.get(2).unwrap());
    let size = util::oop::extract_long(args.get(3).unwrap());
    let value = util::oop::extract_int(args.get(4).unwrap());

    let dest = match obj {
        Oop::Null => offset as *mut libc::c_void,
        _ => unimplemented!(),
    };

    unsafe {
        libc::memset(dest, value, size as usize);
    }

    Ok(None)
}

fn jvm_putChar(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    let dest = util::oop::extract_long(args.get(1).unwrap()) as *mut libc::c_void;
    let value = util::oop::extract_int(args.get(2).unwrap());

    unsafe {
        libc::memset(dest, value, 1);
    }

    Ok(None)
}

fn jvm_copyMemory(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    let _this = args.get(0).unwrap();
    let src_obj = args.get(1).unwrap();
    let src_offset = util::oop::extract_long(args.get(2).unwrap()) as usize;
    let dest_obj = args.get(3).unwrap();
    let dest_offset = util::oop::extract_long(args.get(4).unwrap()) as usize;
    let size = util::oop::extract_long(args.get(5).unwrap()) as usize;

    match src_obj {
        Oop::Null => {
            let v = util::oop::extract_ref(dest_obj);
            let mut v = v.write().unwrap();
            match &mut v.v {
                oop::RefKind::TypeArray(dest_ary) => match dest_ary {
                    oop::TypeArrayValue::Char(dest_ary) => {
                        let dest_ptr = dest_ary.as_mut_ptr() as usize + dest_offset;
                        let dest_ptr = dest_ptr as *mut libc::c_void;

                        let src_ptr = src_offset as *const libc::c_void;
                        unsafe {
                            libc::memcpy(dest_ptr, src_ptr, size);
                        }
                    }
                    t => unimplemented!("t={:?}", t),
                },
                _ => unimplemented!(),
            }
        }
        _ => unreachable!(),
    }

    Ok(None)
}

fn jvm_getChar(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    let ptr = util::oop::extract_long(args.get(1).unwrap()) as *const u16;
    let v = unsafe { *ptr };
    Ok(Some(Oop::new_int(v as i32)))
}
