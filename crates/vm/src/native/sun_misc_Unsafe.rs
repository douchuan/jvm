#![allow(non_snake_case)]

use crate::native::{java_lang_System, new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop;
use crate::oop::{Class, Oop, OopRef};
use crate::runtime::require_class3;
use crate::util;
use classfile::flags::ACC_STATIC;
use std::os::raw::c_void;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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
        new_fn(
            "ensureClassInitialized",
            "(Ljava/lang/Class;)V",
            Box::new(jvm_ensureClassInitialized),
        ),
        new_fn(
            "staticFieldOffset",
            "(Ljava/lang/reflect/Field;)J",
            Box::new(jvm_staticFieldOffset),
        ),
        new_fn(
            "staticFieldBase",
            "(Ljava/lang/reflect/Field;)Ljava/lang/Object;",
            Box::new(jvm_staticFieldBase),
        ),
        new_fn("putByte", "(Ljava/lang/Object;JB)V", Box::new(jvm_putByte)),
        new_fn("getByte", "(Ljava/lang/Object;J)B", Box::new(jvm_getByte2)),
        new_fn("park", "(ZJ)V", Box::new(jvm_park)),
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
    objectFieldOffset(field, false)
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
    let offset = args.get(2).unwrap().extract_long() as usize;
    let size = args.get(3).unwrap().extract_long() as usize;
    let value = args.get(4).unwrap().extract_int();

    match obj {
        Oop::Null => {
            let dest = offset as *mut libc::c_void;
            unsafe {
                libc::memset(dest, value, size as usize);
            }
        }
        Oop::Ref(rf) => {
            let ary = rf.extract_mut_type_array();
            let bytes = ary.extract_mut_bytes();
            unsafe {
                let addr = bytes.as_mut_ptr();
                let addr = addr.add(offset);
                std::ptr::write_bytes(addr, value as u8, size);
            }
        }
        _ => unimplemented!(),
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
            match dest_obj {
                //raw -> raw
                Oop::Null => {
                    let src_ptr = src_offset as *const u8;
                    let dest_ptr = dest_offset as *mut u8;
                    unsafe {
                        std::ptr::copy(src_ptr, dest_ptr, size);
                    }
                }
                //raw -> byte[]
                Oop::Ref(dest) => {
                    let dest = dest.extract_mut_type_array();
                    let dest = dest.extract_mut_bytes();
                    let dest = &mut dest[dest_offset..];
                    let src_ptr = src_offset as *const u8;
                    unsafe {
                        for i in 0..size {
                            let p = src_ptr.add(i);
                            dest[i] = *p;
                        }
                    }
                }
                _ => unimplemented!(),
            }
        }
        Oop::Ref(src) => {
            match dest_obj {
                //byte[] -> raw
                Oop::Null => {
                    let src = src.extract_type_array();
                    let src = src.extract_bytes();
                    let ptr = dest_offset as *mut u8;
                    unsafe {
                        for i in 0..size {
                            let p = ptr.add(i);
                            *p = src[src_offset + i];
                        }
                    }
                }
                //byte[] -> byte[]
                Oop::Ref(dest) => {
                    java_lang_System::arraycopy_diff_obj(
                        src.clone(),
                        src_offset,
                        dest.clone(),
                        dest_offset,
                        size,
                    );
                }
                _ => unimplemented!(),
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

fn jvm_putObject(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let _this = args.get(0).unwrap();
    let o = args.get(1).unwrap();
    let offset = args.get(2).unwrap().extract_long() as usize;
    let x = args.get(3).unwrap();

    let rf = o.extract_ref();
    Class::put_field_value2(rf, offset, x.clone());
    Ok(None)
}

fn jvm_ensureClassInitialized(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let clazz = args.get(1).unwrap();
    let rf = clazz.extract_ref();
    let mirror = rf.extract_mirror();
    let target = mirror.target.clone().unwrap();
    oop::class::init_class(&target);
    oop::class::init_class_fully(&target);
    Ok(None)
}

fn jvm_staticFieldOffset(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let field = args.get(1).unwrap();
    objectFieldOffset(field, true)
}

fn jvm_staticFieldBase(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let field = args.get(1).unwrap();
    let cls = require_class3(None, b"java/lang/reflect/Field").unwrap();
    let cls = cls.get_class();
    let id = cls.get_field_id(&util::S_CLAZZ, &util::S_JAVA_LANG_CLASS, false);
    let v = Class::get_field_value(field.extract_ref(), id);
    Ok(Some(v))
}

fn jvm_putByte(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let _this = args.get(0).unwrap();
    let obj = args.get(1).unwrap();
    let offset = args.get(2).unwrap().extract_long() as usize;
    let x = args.get(3).unwrap().extract_int();

    match obj {
        Oop::Null => {
            let dest = offset as *mut libc::c_void;
            unsafe {
                libc::memset(dest, x, 1);
            }
        }
        Oop::Ref(rf) => {
            let ary = rf.extract_mut_type_array();
            let bytes = ary.extract_mut_bytes();
            bytes[offset] = x as u8;
        }
        t => unimplemented!("{:?}", t),
    }

    Ok(None)
}

fn jvm_getByte2(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let _this = args.get(0).unwrap();
    let obj = args.get(1).unwrap();
    let offset = args.get(2).unwrap().extract_long() as usize;

    let v = match obj {
        Oop::Null => {
            let ptr = offset as *const u8;
            let v = unsafe { *ptr };
            Oop::new_int(v as i32)
        }
        Oop::Ref(rf) => {
            let ary = rf.extract_mut_type_array();
            let bytes = ary.extract_bytes();
            let v = bytes[offset];
            Oop::new_int(v as i32)
        }
        t => unimplemented!("{:?}", t),
    };

    Ok(Some(v))
}

fn jvm_park(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let _this = args.get(0).unwrap();
    let is_absolute = args.get(1).unwrap().extract_int() != 0;
    let time = args.get(2).unwrap().extract_long() as u64;

    if is_absolute {
        let epoch_duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let diff = Duration::from_millis(time) - epoch_duration;
        std::thread::park_timeout(diff);
    } else {
        if time != 0 {
            std::thread::park_timeout(Duration::from_nanos(time));
        } else {
            std::thread::park();
        }
    }

    Ok(None)
}

////////helper

fn objectFieldOffset(field: &Oop, is_static: bool) -> JNIResult {
    let cls = require_class3(None, b"java/lang/reflect/Field").unwrap();

    if is_static {
        let modifier = {
            let cls = cls.get_class();
            let id = cls.get_field_id(&util::S_MODIFIERS, &util::S_I, false);
            let v = Class::get_field_value(field.extract_ref(), id);
            v.extract_int() as u16
        };
        assert_eq!(modifier & ACC_STATIC, ACC_STATIC);
    }

    let slot = {
        let cls = cls.get_class();
        let id = cls.get_field_id(&util::S_SLOT, &util::S_I, false);
        let v = Class::get_field_value(field.extract_ref(), id);
        v.extract_int()
    };

    Ok(Some(Oop::new_long(slot as i64)))
}
