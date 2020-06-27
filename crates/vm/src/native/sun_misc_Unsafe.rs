#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::new_br;
use crate::oop::{Class, Oop, OopRef};
use crate::runtime::require_class3;
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
        new_fn(
            "putObject",
            "(Ljava/lang/Object;JLjava/lang/Object;)V",
            Box::new(jvm_putObject),
        ),
    ]
}

fn jvm_registerNatives(_env: JNIEnv, _args: &Vec<Oop>) -> JNIResult {
    Ok(None)
}

fn jvm_arrayBaseOffset(_env: JNIEnv, _args: &Vec<Oop>) -> JNIResult {
    Ok(Some(Oop::new_int(0)))
}

fn jvm_arrayIndexScale(_env: JNIEnv, _args: &Vec<Oop>) -> JNIResult {
    Ok(Some(Oop::new_int(1)))
}

fn jvm_addressSize(_env: JNIEnv, _args: &Vec<Oop>) -> JNIResult {
    let v = std::mem::size_of::<*mut u8>();
    Ok(Some(Oop::new_int(v as i32)))
}

fn jvm_objectFieldOffset(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let field = args.get(1).unwrap();

    {
        let rf = field.extract_ref();
        let inst = rf.extract_inst();
        let cls = inst.class.clone();
        let cls = cls.get_class();
        debug_assert_eq!(cls.name.as_slice(), b"java/lang/reflect/Field");
    }

    let cls = require_class3(None, b"java/lang/reflect/Field").unwrap();
    let v = {
        let cls = cls.get_class();
        let id = cls.get_field_id(new_br("slot"), new_br("I"), false);
        Class::get_field_value(field.extract_ref(), id)
    };

    let v = v.extract_int();
    Ok(Some(Oop::new_long(v as i64)))
}

// fixme: The semantic requirement here is atomic operation, which needs to be re-implemented here
fn jvm_compareAndSwapObject(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let owner = args.get(1).unwrap();
    let offset = args.get(2).unwrap().extract_long();
    let old_data = args.get(3).unwrap();
    let new_data = args.get(4).unwrap();

    let v_at_offset = Class::get_field_value2(owner.extract_ref(), offset as usize);

    if OopRef::is_eq(&v_at_offset, old_data) {
        Class::put_field_value2(owner.extract_ref(), offset as usize, new_data.clone());
        Ok(Some(Oop::new_int(1)))
    } else {
        Ok(Some(Oop::new_int(0)))
    }
}

fn jvm_getIntVolatile(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let owner = args.get(1).unwrap();
    let offset = args.get(2).unwrap().extract_long();
    let v = Class::get_field_value2(owner.extract_ref(), offset as usize);
    Ok(Some(v))
}

fn jvm_compareAndSwapInt(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let owner = args.get(1).unwrap();
    let offset = args.get(2).unwrap().extract_long();
    let old_data = args.get(3).unwrap().extract_int();
    let new_data = args.get(4).unwrap();

    let v_at_offset = {
        let v = Class::get_field_value2(owner.extract_ref(), offset as usize);
        v.extract_int()
    };

    if v_at_offset == old_data {
        Class::put_field_value2(owner.extract_ref(), offset as usize, new_data.clone());
        Ok(Some(Oop::new_int(1)))
    } else {
        Ok(Some(Oop::new_int(0)))
    }
}

fn jvm_allocateMemory(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let size = args.get(1).unwrap().extract_long() as usize;
    let arr = unsafe { libc::malloc(std::mem::size_of::<u8>() * size) };
    let v = arr as i64;

    Ok(Some(Oop::new_long(v)))
}

fn jvm_freeMemory(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let ptr = args.get(1).unwrap().extract_long() as *mut libc::c_void;

    unsafe {
        libc::free(ptr);
    }

    Ok(None)
}

fn jvm_putLong(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let ptr = args.get(1).unwrap().extract_long() as *mut libc::c_void;
    let l = args.get(2).unwrap().extract_long();
    let v = l.to_be_bytes();
    let v = vec![v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]];
    unsafe {
        libc::memcpy(ptr, v.as_ptr() as *const c_void, 8);
    }

    Ok(None)
}

fn jvm_getByte(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let ptr = args.get(1).unwrap().extract_long() as *const u8;
    let v = unsafe { *ptr };
    Ok(Some(Oop::new_int(v as i32)))
}

fn jvm_compareAndSwapLong(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let owner = args.get(1).unwrap();
    let offset = args.get(2).unwrap().extract_long();
    let old_data = args.get(3).unwrap().extract_long();
    let new_data = args.get(4).unwrap();

    let v_at_offset = {
        let v = Class::get_field_value2(owner.extract_ref(), offset as usize);
        v.extract_long()
    };

    if v_at_offset == old_data {
        Class::put_field_value2(owner.extract_ref(), offset as usize, new_data.clone());
        Ok(Some(Oop::new_int(1)))
    } else {
        Ok(Some(Oop::new_int(0)))
    }
}

fn jvm_getObjectVolatile(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let owner = args.get(1).unwrap();
    let offset = args.get(2).unwrap().extract_long();
    let v_at_offset = Class::get_field_value2(owner.extract_ref(), offset as usize);
    Ok(Some(v_at_offset))
}

fn jvm_pageSize(_env: JNIEnv, _args: &Vec<Oop>) -> JNIResult {
    Ok(Some(Oop::new_int(4 * 1024)))
}

fn jvm_getLongVolatile(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let owner = args.get(1).unwrap();
    let offset = args.get(2).unwrap().extract_long();
    let v_at_offset = Class::get_field_value2(owner.extract_ref(), offset as usize);
    Ok(Some(v_at_offset))
}

fn jvm_setMemory(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let _this = args.get(0).unwrap();
    let obj = args.get(1).unwrap();
    let offset = args.get(2).unwrap().extract_long();
    let size = args.get(3).unwrap().extract_long();
    let value = args.get(4).unwrap().extract_int();

    let dest = match obj {
        Oop::Null => offset as *mut libc::c_void,
        _ => unimplemented!(),
    };

    unsafe {
        libc::memset(dest, value, size as usize);
    }

    Ok(None)
}

fn jvm_putChar(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let dest = args.get(1).unwrap().extract_long() as *mut libc::c_void;
    let value = args.get(2).unwrap().extract_int();

    unsafe {
        libc::memset(dest, value, 1);
    }

    Ok(None)
}

fn jvm_copyMemory(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let _this = args.get(0).unwrap();
    let src_obj = args.get(1).unwrap();
    let src_offset = args.get(2).unwrap().extract_long() as usize;
    let dest_obj = args.get(3).unwrap();
    let dest_offset = args.get(4).unwrap().extract_long() as usize;
    let size = args.get(5).unwrap().extract_long() as usize;

    match src_obj {
        Oop::Null => {
            let rf = dest_obj.extract_ref();
            let dest_ary = rf.extract_mut_type_array();
            let dest_ary = dest_ary.extract_mut_chars();
            let dest_ptr = dest_ary.as_mut_ptr() as usize + dest_offset;
            let dest_ptr = dest_ptr as *mut libc::c_void;

            let src_ptr = src_offset as *const libc::c_void;
            unsafe {
                libc::memcpy(dest_ptr, src_ptr, size);
            }
        }
        _ => unreachable!(),
    }

    Ok(None)
}

fn jvm_getChar(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let arg1 = args.get(1).unwrap();
    let ptr = arg1.extract_long() as *const u16;
    let v = unsafe { *ptr };
    Ok(Some(Oop::new_int(v as i32)))
}

fn jvm_putObject(_env: JNIEnv, _args: &Vec<Oop>) -> JNIResult {
    unimplemented!();
}
