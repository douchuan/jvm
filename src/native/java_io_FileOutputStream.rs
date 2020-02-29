#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{self, Oop};
use crate::runtime::{require_class3, JavaThread};
use crate::util;

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![
        new_fn("initIDs", "()V", Box::new(jvm_initIDs)),
        new_fn("writeBytes", "([BIIZ)V", Box::new(jvm_writeBytes)),
        new_fn("open0", "(Ljava/lang/String;Z)V", Box::new(jvm_open0)),
    ]
}

fn jvm_initIDs(_jt: &mut JavaThread, _env: JNIEnv, _args: Vec<Oop>) -> JNIResult {
    Ok(None)
}

fn jvm_writeBytes(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    let os = args.get(0).unwrap();
    let fd = get_file_descriptor_fd(os.clone());
    let byte_ary = args.get(1).unwrap();
    let off = {
        let v = args.get(2).unwrap();
        util::oop::extract_int(v.clone())
    };
    let len = {
        let v = args.get(3).unwrap();
        util::oop::extract_int(v.clone())
    };
    let append = {
        let v = args.get(4).unwrap();
        util::oop::extract_int(v.clone())
    };

    trace!("append = {}", append);

    let v = util::oop::extract_ref(byte_ary.clone());
    let v = v.lock().unwrap();
    match &v.v {
        oop::RefDesc::TypeArray(ary) => match ary {
            oop::TypeArrayValue::Byte(ary) => {
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
            }
            t => unreachable!("t = {:?}", t),
        },
        _ => unreachable!(),
    }

    Ok(None)
}

fn jvm_open0(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    let this = args.get(0).unwrap();
    let name = util::oop::extract_str(args.get(1).unwrap().clone());
    let append = {
        let v = args.get(2).unwrap();
        util::oop::extract_int(v.clone()) == 1
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

    set_file_descriptor_fd(this.clone(), fd);

    Ok(None)
}

fn get_file_descriptor_fd(fos: Oop) -> i32 {
    let cls = require_class3(None, b"java/io/FileOutputStream").unwrap();
    let fd_this = {
        let cls = cls.lock().unwrap();
        let id = cls.get_field_id(b"fd", b"Ljava/io/FileDescriptor;", false);
        cls.get_field_value(fos.clone(), id)
    };

    let cls = require_class3(None, b"java/io/FileDescriptor").unwrap();
    let fd = {
        let cls = cls.lock().unwrap();
        let id = cls.get_field_id(b"fd", b"I", false);
        cls.get_field_value(fd_this, id)
    };

    util::oop::extract_int(fd)
}

fn set_file_descriptor_fd(fos: Oop, fd: i32) {
    let cls = require_class3(None, b"java/io/FileOutputStream").unwrap();
    let fd_this = {
        let cls = cls.lock().unwrap();
        let id = cls.get_field_id(b"fd", b"Ljava/io/FileDescriptor;", false);
        cls.get_field_value(fos.clone(), id)
    };

    let cls = require_class3(None, b"java/io/FileDescriptor").unwrap();
    let cls = cls.lock().unwrap();
    let id = cls.get_field_id(b"fd", b"I", false);
    cls.put_field_value(fd_this, id, Oop::new_int(fd));
}
