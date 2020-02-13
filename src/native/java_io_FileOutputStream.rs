#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{self, Oop, OopRef};
use crate::runtime::JavaThread;
use std::sync::{Arc, Mutex};

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![
        new_fn("initIDs", "()V", Box::new(jvm_initIDs)),
        new_fn("writeBytes", "([BIIZ)V", Box::new(jvm_writeBytes)),
    ]
}

fn jvm_initIDs(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    Ok(None)
}

fn jvm_writeBytes(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let os = args.get(0).unwrap();
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
    let append = {
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
            Oop::Array(ary) => {
                /*
                ary.elements
                    .iter()
                    .map(|v| {
                        let v = v.lock().unwrap();
                        match v.v {
                            Oop::Int(v) => v as u8,
                            _ => unreachable!()
                        }
                    }).collect()
                */
                let p1 = off as usize;
                let p2 = (off + len) as usize;
                ary.elements[off as usize..(off + len) as usize]
                    .iter()
                    .map(|v| {
                        let v = v.lock().unwrap();
                        match v.v {
                            Oop::Int(v) => v as u8,
                            _ => unreachable!(),
                        }
                    })
                    .collect()
            }
            _ => unreachable!(),
        }
    };

    print!("{}", String::from_utf8_lossy(bytes.as_slice()));

    Ok(None)
}
