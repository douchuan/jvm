#![allow(non_snake_case)]

use crate::native::{self, new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{self, Oop, OopDesc};
use crate::runtime::JavaCall;
use crate::runtime::{self, JavaThread};
use crate::types::OopRef;
use crate::util;
use std::sync::Arc;
use std::time::SystemTime;

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![
        new_fn(
            "arraycopy",
            "(Ljava/lang/Object;ILjava/lang/Object;II)V",
            Box::new(jvm_arraycopy),
        ),
        new_fn("registerNatives", "()V", Box::new(jvm_registerNatives)),
        new_fn(
            "initProperties",
            "(Ljava/util/Properties;)Ljava/util/Properties;",
            Box::new(jvm_initProperties),
        ),
        new_fn("setIn0", "(Ljava/io/InputStream;)V", Box::new(jvm_setIn0)),
        new_fn("setOut0", "(Ljava/io/PrintStream;)V", Box::new(jvm_setOut0)),
        new_fn("setErr0", "(Ljava/io/PrintStream;)V", Box::new(jvm_setErr0)),
        new_fn(
            "mapLibraryName",
            "(Ljava/lang/String;)Ljava/lang/String;",
            Box::new(jvm_mapLibraryName),
        ),
        new_fn(
            "loadLibrary",
            "(Ljava/lang/String;)V",
            Box::new(jvm_loadLibrary),
        ),
        new_fn(
            "identityHashCode",
            "(Ljava/lang/Object;)I",
            Box::new(jvm_identityHashCode),
        ),
        new_fn("nanoTime", "()J", Box::new(jvm_nanoTime)),
        //Note: just for debug
        //        new_fn("getProperty", "(Ljava/lang/String;)Ljava/lang/String;", Box::new(jvm_getProperty)),
    ]
}

fn jvm_registerNatives(_jt: &mut JavaThread, _env: JNIEnv, _args: Vec<OopRef>) -> JNIResult {
    Ok(None)
}

fn jvm_arraycopy(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let src = args.get(0).unwrap();
    let src_pos = {
        let arg1 = args.get(1).unwrap();
        let arg1 = arg1.lock().unwrap();
        match arg1.v {
            Oop::Int(v) => v,
            _ => unreachable!(),
        }
    };
    let dest = args.get(2).unwrap();
    let dest_pos = {
        let arg3 = args.get(3).unwrap();
        let arg3 = arg3.lock().unwrap();
        match arg3.v {
            Oop::Int(v) => v,
            _ => unreachable!(),
        }
    };
    let length = {
        let arg4 = args.get(4).unwrap();
        let arg4 = arg4.lock().unwrap();
        match arg4.v {
            Oop::Int(v) => v,
            _ => unreachable!(),
        }
    };

    //todo: do check & throw exception

    if length == 0 {
        return Ok(None);
    }

    //优化：同一个对象，不可同时上锁，所以需要多一次临时拷贝，对大数组是个考验
    let is_same_obj = Arc::ptr_eq(src, dest);
    if is_same_obj {
        arraycopy_same_obj(
            src.clone(),
            src_pos as usize,
            dest.clone(),
            dest_pos as usize,
            length as usize,
        );
    } else {
        arraycopy_diff_obj(
            src.clone(),
            src_pos as usize,
            dest.clone(),
            dest_pos as usize,
            length as usize,
        );
    }

    Ok(None)
}

fn jvm_initProperties(jt: &mut JavaThread, _env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    //fixme:
    let props = vec![
        ("java.specification.version", "1.8"),
        ("java.specification.name", "Java Platform API Specification"),
        ("java.specification.vendor", "Chuan"),
        ("java.version", "1.8"),
        ("java.vendor", "Chuan"),
        ("java.vendor.url", "https://github.com/douchuan/jvm"),
        ("java.vendor.url.bug", "https://github.com/douchuan/jvm"),
        ("java.class.version", "52.0"),
        ("os.name", "Mac OS X"),
        ("os.version", "18.7.0"),
        ("os.arch", "x86"),
        ("file.separator", util::PATH_DELIMITER_STR),
        ("path.separator", util::PATH_SEP_STR),
        ("line.separator", "\n"),
        ("user.language", "en"),
        ("user.region", "US"),
        ("file.encoding.pkg", "sun.io"),
        ("sun.cpu.isalist", ""),
        //        ("sun.cpu.endian", "little"),
        ("sun.arch.data.model", "32"),
        ("user.name", "chuan"),
        ("user.home", "/Users/douchuan/"),
        ("user.dir", "/Users/douchuan/"),
        ("java.home", "/Users/douchuan/work/prj_rust/jvm/test/"),
        ("file.encoding", "US-ASCII"),
        //        ("java.security.manager", ""),
        //        ("sun.jnu.encoding", "UTF-8"),
        //        ("sun.stdout.encoding", "UTF-8"),
        //        ("sun.stderr.encoding", "UTF-8"),
        //        ("sun.io.unicode.encoding", "UnicodeBig"),
    ];

    let props: Vec<(OopRef, OopRef)> = props
        .iter()
        .map(|(k, v)| {
            let k = util::oop::new_java_lang_string2(jt, k);
            let v = util::oop::new_java_lang_string2(jt, v);

            (k, v)
        })
        .collect();

    let props_oop = args.get(0).unwrap();
    let cls = {
        let v = props_oop.lock().unwrap();
        match &v.v {
            Oop::Inst(inst) => inst.class.clone(),
            _ => unreachable!(),
        }
    };

    let mir = {
        let cls = cls.lock().unwrap();
        let id = util::new_method_id(
            b"put",
            b"(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
        );
        cls.get_virtual_method(id).unwrap()
    };

    for it in props.iter() {
        let args = vec![props_oop.clone(), it.0.clone(), it.1.clone()];

        let mut jc = JavaCall::new_with_args(jt, mir.clone(), args);
        let mut stack = runtime::Stack::new(1);
        jc.invoke(jt, &mut stack, false);

        //fixme: should be removed
        if jt.is_meet_ex() {
            unreachable!("jvm_initProperties meet ex");
        }
    }

    Ok(Some(props_oop.clone()))
}

fn jvm_setIn0(_jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let v = args.get(0).unwrap();
    let cls = { env.lock().unwrap().class.clone() };
    let mut cls = cls.lock().unwrap();
    let id = cls.get_field_id(b"in", b"Ljava/io/InputStream;", true);
    cls.put_static_field_value(id, v.clone());
    Ok(None)
}

fn jvm_setOut0(_jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let v = args.get(0).unwrap();
    let cls = { env.lock().unwrap().class.clone() };
    let mut cls = cls.lock().unwrap();
    let id = cls.get_field_id(b"out", b"Ljava/io/PrintStream;", true);
    cls.put_static_field_value(id, v.clone());
    Ok(None)
}

fn jvm_setErr0(_jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let v = args.get(0).unwrap();
    let cls = { env.lock().unwrap().class.clone() };
    let mut cls = cls.lock().unwrap();
    let id = cls.get_field_id(b"err", b"Ljava/io/PrintStream;", true);
    cls.put_static_field_value(id, v.clone());
    Ok(None)
}

fn jvm_mapLibraryName(jt: &mut JavaThread, _env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let v = args.get(0).unwrap();
    let s = util::oop::extract_str(v.clone());

    trace!("mapLibraryName libname = {}", s);
    let mut name = String::new();
    if cfg!(target_os = "macos") {
        name.push_str("lib");
        name.push_str(&s);
        name.push_str(".dylib");
    } else if cfg!(target_os = "windows") {
        unimplemented!();
    //        name.extend_from_slice(s.as_bytes());
    //        name.extend_from_slice(".dll".as_bytes());
    } else if cfg!(target_os = "linux") {
        unimplemented!();
    //        name.extend_from_slice("lib".as_bytes());
    //        name.extend_from_slice(s.as_bytes());
    //        name.extend_from_slice(".so".as_bytes());
    } else {
        unimplemented!()
    }
    trace!("mapLibraryName name = {}", name);

    let v = util::oop::new_java_lang_string2(jt, &name);

    Ok(Some(v))
}

fn jvm_loadLibrary(_jt: &mut JavaThread, _env: JNIEnv, _args: Vec<OopRef>) -> JNIResult {
    Ok(None)
}

fn jvm_identityHashCode(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    native::java_lang_Object::jvm_hashCode(jt, env, args)
}

/*
fn jvm_getProperty(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let key = args.get(0).unwrap();

    let str_key = util::oop::extract_str(key.clone());
    warn!("xxxx jvm_getProperty key = {}", str_key);

    let cls = require_class3(None, b"java/lang/System").unwrap();
    let props = {
        let cls = cls.lock().unwrap();
        let id = cls.get_field_id(b"props", b"Ljava/util/Properties;", true);
        cls.get_static_field_value(id)
    };

    let prop_cls = require_class3(None, b"java/util/Properties").unwrap();
    let mir = {
        let cls = prop_cls.lock().unwrap();
        let id = util::new_method_id(b"getProperty", b"(Ljava/lang/String;)Ljava/lang/String;");
        cls.get_class_method(id).unwrap()
    };

    let args = vec![props, key.clone()];
    let mut stack = Stack::new(1);
    let mut jc = runtime::java_call::JavaCall::new_with_args(jt, mir, args);
    jc.invoke(jt, &mut stack, false);

    let v = stack.pop_ref();

    //    trace!("xxxxx 1, str_key = {}", String::from_utf8_lossy(str_key.as_slice()));
    //    let str_v = util::oop::extract_str(v.clone());
    //    warn!("xxxx jvm_getProperty v = {}", String::from_utf8_lossy(str_v.as_slice()));
    //    trace!("xxxxx 2");

    Ok(Some(v))
}
*/

/*
同一个对象不可同时lock，需要tmp buf做中转
*/
fn arraycopy_same_obj(src: OopRef, src_pos: usize, dest: OopRef, dest_pos: usize, length: usize) {
    let is_type_ary = {
        let src = src.lock().unwrap();
        match &src.v {
            Oop::TypeArray(_) => true,
            _ => unreachable!(),
        }
    };

    if is_type_ary {
        let tmp = {
            let src = src.lock().unwrap();

            let mut tmp = Vec::with_capacity(length);
            //just choose the needed region
            match &src.v {
                Oop::TypeArray(s) => match s {
                    oop::TypeArrayValue::Char(ary) => {
                        ary[src_pos..(src_pos + length)].iter().for_each(|v| {
                            tmp.push(*v);
                        });
                    }
                    oop::TypeArrayValue::Byte(ary) => {
                        ary[src_pos..(src_pos + length)].iter().for_each(|v| {
                            tmp.push(*v as u16);
                        });
                    }
                    t => unreachable!("t = {:?}", t),
                },
                _ => unreachable!(),
            }

            tmp
        };

        let mut dest = dest.lock().unwrap();
        match &mut dest.v {
            Oop::TypeArray(ary) => match ary {
                oop::TypeArrayValue::Char(dest) => {
                    dest[dest_pos..(dest_pos + length)].clone_from_slice(&tmp[..]);
                }
                oop::TypeArrayValue::Byte(dest) => {
                    let src: Vec<u8> = tmp.iter().map(|v| *v as u8).collect();
                    dest[dest_pos..(dest_pos + length)].clone_from_slice(&src[..]);
                }
                t => unreachable!("t = {:?}", t),
            },

            _ => unreachable!(),
        }
    } else {
        let tmp = {
            let ary = src.lock().unwrap();
            match &ary.v {
                Oop::Array(ary) => {
                    let mut tmp = Vec::with_capacity(length);
                    tmp.clone_from_slice(&ary.elements[src_pos..(src_pos + length)]);
                    tmp
                }
                _ => unreachable!(),
            }
        };

        let mut dest = dest.lock().unwrap();
        match &mut dest.v {
            Oop::Array(ary) => {
                ary.elements[dest_pos..(dest_pos + length)].clone_from_slice(&tmp[..]);
            }
            _ => unreachable!(),
        }
    }
}

fn arraycopy_diff_obj(src: OopRef, src_pos: usize, dest: OopRef, dest_pos: usize, length: usize) {
    let src = src.lock().unwrap();
    let mut dest = dest.lock().unwrap();

    let is_type_ary = {
        match &src.v {
            Oop::TypeArray(_) => true,
            _ => unreachable!(),
        }
    };

    if is_type_ary {
        match &src.v {
            Oop::TypeArray(src_ary) => match src_ary {
                oop::TypeArrayValue::Char(src_ary) => match &mut dest.v {
                    Oop::TypeArray(dest_ary) => match dest_ary {
                        oop::TypeArrayValue::Char(dest_ary) => {
                            dest_ary[dest_pos..(dest_pos + length)]
                                .clone_from_slice(&src_ary[src_pos..(src_pos + length)]);
                        }
                        _ => unreachable!(),
                    },
                    _ => unreachable!(),
                },
                oop::TypeArrayValue::Byte(src_ary) => match &mut dest.v {
                    Oop::TypeArray(dest_ary) => match dest_ary {
                        oop::TypeArrayValue::Byte(dest_ary) => {
                            dest_ary[dest_pos..(dest_pos + length)]
                                .clone_from_slice(&src_ary[src_pos..(src_pos + length)]);
                        }
                        _ => unreachable!(),
                    },
                    _ => unreachable!(),
                },

                t => unreachable!("t = {:?}", t),
            },
            _ => unreachable!(),
        }
    } else {
        match &src.v {
            Oop::Array(src) => match &mut dest.v {
                Oop::Array(dest) => {
                    dest.elements[dest_pos as usize..(dest_pos + length) as usize]
                        .clone_from_slice(
                            &src.elements[src_pos as usize..(src_pos + length) as usize],
                        );
                }
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }
}

fn jvm_nanoTime(_jt: &mut JavaThread, _env: JNIEnv, _args: Vec<OopRef>) -> JNIResult {
    let v = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_nanos(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    };

    Ok(Some(OopDesc::new_long(v as i64)))
}
