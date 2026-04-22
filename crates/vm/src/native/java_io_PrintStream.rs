#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::Oop;

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![
        // println variants
        new_fn("println", "(I)V", Box::new(jvm_println_int)),
        new_fn("println", "(J)V", Box::new(jvm_println_long)),
        new_fn("println", "(F)V", Box::new(jvm_println_float)),
        new_fn("println", "(D)V", Box::new(jvm_println_double)),
        new_fn("println", "(Z)V", Box::new(jvm_println_bool)),
        new_fn("println", "(C)V", Box::new(jvm_println_char)),
        new_fn(
            "println",
            "(Ljava/lang/String;)V",
            Box::new(jvm_println_string),
        ),
        new_fn(
            "println",
            "(Ljava/lang/Object;)V",
            Box::new(jvm_println_object),
        ),
        new_fn("println", "()V", Box::new(jvm_println_void)),
        // print variants
        new_fn("print", "(I)V", Box::new(jvm_print_int)),
        new_fn("print", "(Ljava/lang/String;)V", Box::new(jvm_print_string)),
        new_fn("print", "(Ljava/lang/Object;)V", Box::new(jvm_print_object)),
        // write variants
        new_fn("write", "(I)V", Box::new(jvm_write_byte)),
        new_fn("write", "([B)V", Box::new(jvm_write_bytes_all)),
        new_fn("write", "([BII)V", Box::new(jvm_write_bytes)),
        // flush/close
        new_fn("flush", "()V", Box::new(jvm_flush)),
        new_fn("close", "()V", Box::new(jvm_close)),
    ]
}

fn jvm_println_int(_env: JNIEnv, args: &[Oop]) -> JNIResult {
    let val = args.get(1).unwrap().extract_int();
    println!("{}", val);
    Ok(None)
}

fn jvm_println_long(_env: JNIEnv, args: &[Oop]) -> JNIResult {
    let val = args.get(1).unwrap().extract_long();
    println!("{}", val);
    Ok(None)
}

fn jvm_println_float(_env: JNIEnv, args: &[Oop]) -> JNIResult {
    let val = args.get(1).unwrap().extract_float();
    println!("{}", val);
    Ok(None)
}

fn jvm_println_double(_env: JNIEnv, args: &[Oop]) -> JNIResult {
    let val = args.get(1).unwrap().extract_double();
    println!("{}", val);
    Ok(None)
}

fn jvm_println_bool(_env: JNIEnv, args: &[Oop]) -> JNIResult {
    let val = args.get(1).unwrap().extract_int();
    println!("{}", val != 0);
    Ok(None)
}

fn jvm_println_char(_env: JNIEnv, args: &[Oop]) -> JNIResult {
    let val = args.get(1).unwrap().extract_int();
    if let Some(c) = char::from_u32(val as u32) {
        println!("{}", c);
    }
    Ok(None)
}

fn jvm_println_string(_env: JNIEnv, args: &[Oop]) -> JNIResult {
    let s = args.get(1).unwrap();
    if s.is_null() {
        println!("null");
    } else {
        let s = Oop::java_lang_string(s.extract_ref());
        println!("{}", s);
    }
    Ok(None)
}

fn jvm_println_object(_env: JNIEnv, args: &[Oop]) -> JNIResult {
    let obj = args.get(1).unwrap();
    if obj.is_null() {
        println!("null");
    } else {
        let cls = crate::oop::with_heap(|heap| {
            let rf = obj.extract_ref();
            let desc = heap.get(rf);
            let guard = desc.read().unwrap();
            guard.v.extract_inst().class.clone()
        });
        let cls_name = String::from_utf8_lossy(cls.get_class().name.as_slice());
        println!("{}", cls_name.replace('/', "."));
    }
    Ok(None)
}

fn jvm_println_void(_env: JNIEnv, _args: &[Oop]) -> JNIResult {
    println!();
    Ok(None)
}

fn jvm_print_int(_env: JNIEnv, args: &[Oop]) -> JNIResult {
    let val = args.get(1).unwrap().extract_int();
    print!("{}", val);
    Ok(None)
}

fn jvm_print_string(_env: JNIEnv, args: &[Oop]) -> JNIResult {
    let s = args.get(1).unwrap();
    if s.is_null() {
        print!("null");
    } else {
        let s = Oop::java_lang_string(s.extract_ref());
        print!("{}", s);
    }
    Ok(None)
}

fn jvm_print_object(_env: JNIEnv, args: &[Oop]) -> JNIResult {
    let obj = args.get(1).unwrap();
    if obj.is_null() {
        print!("null");
    } else {
        let cls = crate::oop::with_heap(|heap| {
            let rf = obj.extract_ref();
            let desc = heap.get(rf);
            let guard = desc.read().unwrap();
            guard.v.extract_inst().class.clone()
        });
        let cls_name = String::from_utf8_lossy(cls.get_class().name.as_slice());
        print!("{}", cls_name.replace('/', "."));
    }
    Ok(None)
}

fn jvm_write_byte(_env: JNIEnv, args: &[Oop]) -> JNIResult {
    let val = args.get(1).unwrap().extract_int();
    if let Some(b) = (val as u8).checked_sub(0) {
        print!("{}", b as char);
    }
    Ok(None)
}

fn jvm_write_bytes_all(_env: JNIEnv, args: &[Oop]) -> JNIResult {
    let ary = args.get(1).unwrap();
    if !ary.is_null() {
        let rf = ary.extract_ref();
        let bytes: Vec<u8> = crate::oop::with_heap(|heap| {
            let desc = heap.get(rf);
            let guard = desc.read().unwrap();
            match &guard.v {
                crate::oop::RefKind::TypeArray(crate::oop::TypeArrayDesc::Byte(b)) => (**b).clone(),
                _ => vec![],
            }
        });
        print!("{}", String::from_utf8_lossy(&bytes));
    }
    Ok(None)
}

fn jvm_write_bytes(_env: JNIEnv, args: &[Oop]) -> JNIResult {
    let ary = args.get(1).unwrap();
    let offset = args.get(2).unwrap().extract_int() as usize;
    let len = args.get(3).unwrap().extract_int() as usize;
    if !ary.is_null() {
        let rf = ary.extract_ref();
        let bytes: Vec<u8> = crate::oop::with_heap(|heap| {
            let desc = heap.get(rf);
            let guard = desc.read().unwrap();
            match &guard.v {
                crate::oop::RefKind::TypeArray(crate::oop::TypeArrayDesc::Byte(b)) => {
                    b[offset..offset + len].to_vec()
                }
                _ => vec![],
            }
        });
        print!("{}", String::from_utf8_lossy(&bytes));
    }
    Ok(None)
}

fn jvm_flush(_env: JNIEnv, _args: &[Oop]) -> JNIResult {
    Ok(None)
}

fn jvm_close(_env: JNIEnv, _args: &[Oop]) -> JNIResult {
    Ok(None)
}
