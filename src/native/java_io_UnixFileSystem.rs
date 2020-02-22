#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{Oop, OopDesc};
use crate::runtime::JavaThread;
use crate::types::OopRef;
use crate::util;
use std::fs;

//FileSystem.java define
const BA_EXISTS: i32        = 0x01;
const BA_REGULAR: i32       = 0x02;
const BA_DIRECTORY: i32     = 0x04;
const _BA_HIDDEN: i32       = 0x08;
const ACCESS_READ: i32      = 0x04;
const ACCESS_WRITE: i32     = 0x02;
const ACCESS_EXECUTE: i32   = 0x01;

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![
        new_fn("initIDs", "()V", Box::new(jvm_initIDs)),
        new_fn(
            "getBooleanAttributes0",
            "(Ljava/io/File;)I",
            Box::new(jvm_getBooleanAttributes0),
        ),
        new_fn("checkAccess", "(Ljava/io/File;I)Z", Box::new(jvm_checkAccess)),
    ]
}

fn jvm_initIDs(_jt: &mut JavaThread, _env: JNIEnv, _args: Vec<OopRef>) -> JNIResult {
    Ok(None)
}

fn jvm_getBooleanAttributes0(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let file = args.get(1).unwrap();
    let path = {
        let cls = {
            let v = file.lock().unwrap();
            match &v.v {
                Oop::Inst(inst) => inst.class.clone(),
                _ => unreachable!(),
            }
        };

        let cls = cls.lock().unwrap();
        let fir = cls.get_field_id(b"path", b"Ljava/lang/String;", false);
        cls.get_field_value(file.clone(), fir)
    };
    let path = util::oop::extract_str(path);

    let mut r = 0;
    match fs::metadata(path) {
        Ok(attr) => {
            r |= BA_EXISTS;
            if attr.is_file() {
                r |= BA_REGULAR;
            }
            if attr.is_dir() {
                r |= BA_DIRECTORY;
            }
        }
        _ => (),
    }

    Ok(Some(OopDesc::new_int(r)))
}

fn jvm_checkAccess(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let file = args.get(1).unwrap();
    let path = {
        let cls = {
            let v = file.lock().unwrap();
            match &v.v {
                Oop::Inst(inst) => inst.class.clone(),
                _ => unreachable!(),
            }
        };

        let cls = cls.lock().unwrap();
        let fir = cls.get_field_id(b"path", b"Ljava/lang/String;", false);
        cls.get_field_value(file.clone(), fir)
    };
    let path = util::oop::extract_str(path);
    let access = args.get(2).unwrap();
    let access = {
        let v = access.lock().unwrap();
        match v.v {
            Oop::Int(v) => v,
            _ => unreachable!()
        }
    };

    let mut amode = 0;
    if (access & ACCESS_READ) == ACCESS_READ {
        amode |= libc::R_OK;
    }
    if (access & ACCESS_WRITE) == ACCESS_WRITE {
        amode |= libc::W_OK;
    }
    if (access & ACCESS_EXECUTE) == ACCESS_EXECUTE {
        amode |= libc::X_OK;
    }

    let r = unsafe {
        use std::ffi::CString;
        let path = CString::new(path).unwrap();
        if libc::access(path.as_ptr(), amode) == 0 {
            1
        } else {
            0
        }
    };

    Ok(Some(OopDesc::new_int(r)))
}
