#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::OopDesc;
use crate::runtime::{require_class3, JavaThread};
use crate::types::OopRef;
use crate::util;
use std::fs;

//FileSystem.java define
const BA_EXISTS: i32 = 0x01;
const BA_REGULAR: i32 = 0x02;
const BA_DIRECTORY: i32 = 0x04;
const _BA_HIDDEN: i32 = 0x08;
const ACCESS_READ: i32 = 0x04;
const ACCESS_WRITE: i32 = 0x02;
const ACCESS_EXECUTE: i32 = 0x01;

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![
        new_fn("initIDs", "()V", Box::new(jvm_initIDs)),
        new_fn(
            "getBooleanAttributes0",
            "(Ljava/io/File;)I",
            Box::new(jvm_getBooleanAttributes0),
        ),
        new_fn(
            "checkAccess",
            "(Ljava/io/File;I)Z",
            Box::new(jvm_checkAccess),
        ),
        new_fn(
            "canonicalize0",
            "(Ljava/lang/String;)Ljava/lang/String;",
            Box::new(jvm_canonicalize0),
        ),
        new_fn(
            "createFileExclusively",
            "(Ljava/lang/String;)Z",
            Box::new(jvm_createFileExclusively),
        ),
    ]
}

fn jvm_initIDs(_jt: &mut JavaThread, _env: JNIEnv, _args: Vec<OopRef>) -> JNIResult {
    Ok(None)
}

fn jvm_getBooleanAttributes0(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let file = args.get(1).unwrap();
    let path = get_File_path(file.clone());

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
    let path = get_File_path(file.clone());

    let access = args.get(2).unwrap();
    let access = util::oop::extract_int(access.clone());

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

fn jvm_canonicalize0(jt: &mut JavaThread, _env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let path = util::oop::extract_str(args.get(1).unwrap().clone());
    let path = std::path::Path::new(&path);
    let path = path.canonicalize().expect("path canonicalize failed");
    let path = path.to_str().expect("path to_str failed");
    let path = util::oop::new_java_lang_string2(jt, path);

    Ok(Some(path))
}

fn jvm_createFileExclusively(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let path = args.get(1).unwrap();
    let path = util::oop::extract_str(path.clone());
    let v = match std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&path)
    {
        Ok(_) => 1,
        Err(e) => {
            error!("open {}, error = {:?}", path, e);
            0
        }
    };
    Ok(Some(OopDesc::new_int(v)))
}

fn get_File_path(file: OopRef) -> String {
    let cls = require_class3(None, b"java/io/File").unwrap();
    let path = {
        let cls = cls.lock().unwrap();
        let fir = cls.get_field_id(b"path", b"Ljava/lang/String;", false);
        cls.get_field_value(file.clone(), fir)
    };
    util::oop::extract_str(path)
}
