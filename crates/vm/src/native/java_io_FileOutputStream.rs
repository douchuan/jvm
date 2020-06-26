#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{Class, Oop, OopRef};
use crate::runtime::require_class3;
use crate::util;

static mut FILE_OUTPUT_STREAM_FD: usize = 0;
static mut FILE_DESCRIPTOR_FD: usize = 0;

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![
        new_fn("initIDs", "()V", Box::new(jvm_initIDs)),
        new_fn("writeBytes", "([BIIZ)V", Box::new(jvm_writeBytes)),
        new_fn("open0", "(Ljava/lang/String;Z)V", Box::new(jvm_open0)),
    ]
}

fn jvm_initIDs(_env: JNIEnv, _args: &Vec<Oop>) -> JNIResult {
    //setup: java.io.FileOutputStream fd
    let cls = require_class3(None, b"java/io/FileOutputStream").unwrap();
    let cls = cls.get_class();
    let id = cls.get_field_id(util::S_FD.clone(), util::S_JAVA_IO_FD.clone(), false);
    unsafe {
        FILE_OUTPUT_STREAM_FD = id.offset;
    }

    //setup: java.io.FileDescriptor fd
    let cls = require_class3(None, b"java/io/FileDescriptor").unwrap();
    let cls = cls.get_class();
    let id = cls.get_field_id(util::S_FD.clone(), util::S_I.clone(), false);
    unsafe {
        FILE_DESCRIPTOR_FD = id.offset;
    }

    Ok(None)
}

fn jvm_writeBytes(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let os = args.get(0).unwrap();
    let fd = get_file_descriptor_fd(os);
    let byte_ary = args.get(1).unwrap();
    let off = args.get(2).unwrap().extract_int();
    let len = args.get(3).unwrap().extract_int();
    let append = args.get(4).unwrap().extract_int();

    let rf = byte_ary.extract_ref();
    let ary = rf.extract_type_array();
    let ary = ary.extract_bytes();
    let (_, ary) = ary.split_at(off as usize);
    let len = len as usize;

    unsafe {
        if append == 1 {
            libc::lseek(fd, 0, libc::SEEK_END);
        }

        if -1 == libc::write(fd, ary.as_ptr() as *const libc::c_void, len) {
            panic!("write failed");
        }
    }

    Ok(None)
}

fn jvm_open0(_env: JNIEnv, args: &Vec<Oop>) -> JNIResult {
    let this = args.get(0).unwrap();
    let name = args.get(1).unwrap();
    let name = OopRef::java_lang_string(name.extract_ref());
    let append = {
        let v = args.get(2).unwrap().extract_int();
        v == 1
    };
    let fd = unsafe {
        use std::ffi::CString;
        let name = CString::new(name).unwrap();
        let mut flag = libc::O_WRONLY | libc::O_CREAT;
        if append {
            flag |= libc::O_APPEND;
        } else {
            flag |= libc::O_TRUNC;
        }
        libc::open(name.as_ptr(), flag)
    };

    set_file_descriptor_fd(this, fd);

    Ok(None)
}

fn get_file_descriptor_fd(fos: &Oop) -> i32 {
    let fd_this = {
        let offset = unsafe { FILE_OUTPUT_STREAM_FD };
        Class::get_field_value2(fos.extract_ref(), offset)
    };

    let fd = {
        let offset = unsafe { FILE_DESCRIPTOR_FD };
        Class::get_field_value2(fd_this.extract_ref(), offset)
    };

    fd.extract_int()
}

fn set_file_descriptor_fd(fos: &Oop, fd: i32) {
    let fd_this = {
        let offset = unsafe { FILE_OUTPUT_STREAM_FD };
        Class::get_field_value2(fos.extract_ref(), offset)
    };

    let offset = unsafe { FILE_DESCRIPTOR_FD };
    Class::put_field_value2(fd_this.extract_ref(), offset, Oop::new_int(fd));
}
