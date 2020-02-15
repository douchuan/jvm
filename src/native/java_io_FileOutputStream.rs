#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{self, Oop};
use crate::runtime::JavaThread;
use crate::types::OopRef;

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![
        new_fn("initIDs", "()V", Box::new(jvm_initIDs)),
        new_fn("writeBytes", "([BIIZ)V", Box::new(jvm_writeBytes)),
    ]
}

fn jvm_initIDs(_jt: &mut JavaThread, _env: JNIEnv, _args: Vec<OopRef>) -> JNIResult {
    Ok(None)
}

fn jvm_writeBytes(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let _os = args.get(0).unwrap();
    let byte_ary = args.get(1).unwrap();
    let off = {
        let v = args.get(2).unwrap();
        let v = v.lock().unwrap();
        match v.v {
            Oop::Int(v) => v,
            _ => unreachable!(),
        }
    };
    let len = {
        let v = args.get(3).unwrap();
        let v = v.lock().unwrap();
        match v.v {
            Oop::Int(v) => v,
            _ => unreachable!(),
        }
    };
    let _append = {
        let v = args.get(4).unwrap();
        let v = v.lock().unwrap();
        match v.v {
            Oop::Int(v) => v,
            _ => unreachable!(),
        }
    };

    let bytes: Vec<u8> = {
        let v = byte_ary.lock().unwrap();
        match &v.v {
            Oop::TypeArray(ary) => match ary {
                oop::TypeArrayValue::Byte(ary) => ary[off as usize..(off + len) as usize]
                    .iter()
                    .map(|v| *v)
                    .collect(),
                t => unreachable!("t = {:?}", t),
            },
            _ => unreachable!(),
        }
    };

    //    println!("xxxxxx 1");
    print!("{}", String::from_utf8_lossy(bytes.as_slice()));
    //    print!("{:?}, len={}", bytes.as_slice(), bytes.len());
    //    io::stdout().flush().unwrap();
    //    print!("{}", String::from_utf8_lossy("\n".as_bytes()));

    Ok(None)
}
