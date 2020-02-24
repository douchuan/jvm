#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{Oop, OopDesc, TypeArrayValue};
use crate::runtime::{require_class3, JavaThread};
use crate::types::OopRef;
use crate::util;

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![
        new_fn("initIDs", "()V", Box::new(jvm_initIDs)),
        new_fn("open0", "(Ljava/lang/String;)V", Box::new(jvm_open0)),
        new_fn("readBytes", "([BII)I", Box::new(jvm_readBytes)),
        new_fn("available0", "()I", Box::new(jvm_available0)),
        new_fn("close0", "()V", Box::new(jvm_close0)),
    ]
}

fn jvm_initIDs(_jt: &mut JavaThread, _env: JNIEnv, _args: Vec<OopRef>) -> JNIResult {
    Ok(None)
}

fn jvm_open0(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let this = args.get(0).unwrap();
    let name = {
        let v = args.get(1).unwrap();
        util::oop::extract_str(v.clone())
    };
    let fd = unsafe {
        use std::ffi::CString;
        let name = CString::new(name).unwrap();
        libc::open(name.as_ptr(), libc::O_RDONLY)
    };

    set_file_descriptor_fd(this.clone(), fd);

    Ok(None)
}

fn jvm_readBytes(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let this = args.get(0).unwrap();
    let fd = get_file_descriptor_fd(this.clone());
    let byte_ary = args.get(1).unwrap();
    let off = {
        let v = args.get(2).unwrap();
        util::oop::extract_int(v.clone())
    };
    let len = {
        let v = args.get(3).unwrap();
        util::oop::extract_int(v.clone())
    };

    let mut byte_ary = byte_ary.lock().unwrap();
    let n = match &mut byte_ary.v {
        Oop::TypeArray(ary) => match ary {
            TypeArrayValue::Byte(ary) => {
                let (_, ptr) = ary.split_at_mut(off as usize);
                unsafe { libc::read(fd, ptr.as_mut_ptr() as *mut libc::c_void, len as usize) }
            }
            _ => unreachable!(),
        },
        _ => unreachable!(),
    };

    Ok(Some(OopDesc::new_int(n as i32)))
}

fn jvm_available0(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let this = args.get(0).unwrap();
    let fd = get_file_descriptor_fd(this.clone());

    if fd == -1 {
        unimplemented!("Stream Closed");
    }

    let mut size = -1i64;
    let mut current = -1i64;

    unsafe {
        let mut stat: libc::stat = std::mem::zeroed();
        if libc::fstat(fd, &mut stat) != -1 {
            let mode = stat.st_mode;
            if (mode & libc::S_IFIFO == libc::S_IFIFO)
                || (mode & libc::S_IFCHR == libc::S_IFCHR)
                || (mode & libc::S_IFSOCK == libc::S_IFSOCK)
            {
                let mut n = 0;
                if libc::ioctl(fd, libc::FIONREAD, &mut n) >= 0 {
                    return Ok(Some(OopDesc::new_int(n)));
                }
            } else if mode & libc::S_IFREG == libc::S_IFREG {
                size = stat.st_size;
            }
        }

        current = libc::lseek(fd, 0, libc::SEEK_CUR);
        if current == -1 {
            return Ok(Some(OopDesc::new_int(0)));
        }

        if size < current {
            size = libc::lseek(fd, 0, libc::SEEK_END);
            if size == -1 {
                return Ok(Some(OopDesc::new_int(0)));
            }

            if libc::lseek(fd, current, libc::SEEK_SET) == -1 {
                return Ok(Some(OopDesc::new_int(0)));
            }
        }
    }

    return Ok(Some(OopDesc::new_int((size - current) as i32)));
}

fn jvm_close0(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let this = args.get(0).unwrap();
    let fd = get_file_descriptor_fd(this.clone());
    unsafe {
        libc::close(fd);
    }
    Ok(None)
}

fn set_file_descriptor_fd(fin: OopRef, fd: i32) {
    let cls = require_class3(None, b"java/io/FileInputStream").unwrap();
    let fd_this = {
        let cls = cls.lock().unwrap();
        let id = cls.get_field_id(b"fd", b"Ljava/io/FileDescriptor;", false);
        cls.get_field_value(fin.clone(), id)
    };

    let cls = require_class3(None, b"java/io/FileDescriptor").unwrap();
    let cls = cls.lock().unwrap();
    let id = cls.get_field_id(b"fd", b"I", false);
    cls.put_field_value(fd_this, id, OopDesc::new_int(fd));
}

fn get_file_descriptor_fd(fin: OopRef) -> i32 {
    let cls = require_class3(None, b"java/io/FileInputStream").unwrap();
    let fd_this = {
        let cls = cls.lock().unwrap();
        let id = cls.get_field_id(b"fd", b"Ljava/io/FileDescriptor;", false);
        cls.get_field_value(fin.clone(), id)
    };

    let cls = require_class3(None, b"java/io/FileDescriptor").unwrap();
    let fd = {
        let cls = cls.lock().unwrap();
        let id = cls.get_field_id(b"fd", b"I", false);
        cls.get_field_value(fd_this, id)
    };

    util::oop::extract_int(fd)
}
