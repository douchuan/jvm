#![allow(non_snake_case)]

use crate::native::{self, new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{self, Oop, OopRefDesc, RefDesc};
use crate::runtime::JavaCall;
use crate::runtime::{self, JavaThread};
use crate::util;
use std::sync::{Arc, Mutex};
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
        new_fn("currentTimeMillis", "()J", Box::new(jvm_currentTimeMillis)),
        //Note: just for debug
        //        new_fn("getProperty", "(Ljava/lang/String;)Ljava/lang/String;", Box::new(jvm_getProperty)),
    ]
}

fn jvm_registerNatives(_jt: &mut JavaThread, _env: JNIEnv, _args: Vec<Oop>) -> JNIResult {
    Ok(None)
}

fn jvm_arraycopy(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    let src = args.get(0).unwrap();
    let src_pos = util::oop::extract_int(args.get(1).unwrap().clone());
    let dest = args.get(2).unwrap();
    let dest_pos = util::oop::extract_int(args.get(3).unwrap().clone());
    let length = util::oop::extract_int(args.get(4).unwrap().clone());

    //todo: do check & throw exception

    if length == 0 {
        return Ok(None);
    }

    let src_ref = util::oop::extract_ref(src.clone());
    let dest_ref = util::oop::extract_ref(dest.clone());
    let is_same_obj = Arc::ptr_eq(&src_ref, &dest_ref);

    if is_same_obj {
        arraycopy_same_obj(
            src_ref,
            src_pos as usize,
            dest_ref,
            dest_pos as usize,
            length as usize,
        );
    } else {
        arraycopy_diff_obj(
            src_ref,
            src_pos as usize,
            dest_ref,
            dest_pos as usize,
            length as usize,
        );
    }

    Ok(None)
}

fn jvm_initProperties(jt: &mut JavaThread, _env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    //fixme:
    let props = vec![
        ("file.encoding.pkg", "sun.io"),
        ("file.encoding", "US-ASCII"),
        ("file.separator", util::FILE_SEP),
        ("java.class.path", "."),
        ("java.class.version", "52.0"),
        ("java.security.egd", "file:/dev/random"),
        // ("java.security.debug", "all"),
        // ("java.security.auth.debug", "all"),
        ("java.specification.version", "1.8"),
        ("java.specification.name", "Java Platform API Specification"),
        ("java.specification.vendor", "Chuan"),
        ("java.vendor", "Chuan"),
        ("java.vendor.url", "https://github.com/douchuan/jvm"),
        ("java.vendor.url.bug", "https://github.com/douchuan/jvm"),
        ("java.version", "1.8"),
        ("line.separator", util::LINE_SEP),
        ("os.arch", "x86_64"),
        ("os.name", "Mac OS X"),
        ("os.version", "18.7.0"),
        ("path.separator", util::PATH_SEP),
        ("sun.arch.data.model", "64"),
        ("sun.cpu.endian", "little"),
        ("sun.cpu.isalist", ""),
        // ("sun.misc.URLClassPath.debug", "true"),
        // ("sun.misc.URLClassPath.debugLookupCache", "true"),
        ("user.language", "en"),
        ("user.name", "chuan"),
        ("user.region", "US"),
        //        ("java.security.manager", ""),
        //        ("sun.jnu.encoding", "UTF-8"),
        //        ("sun.stdout.encoding", "UTF-8"),
        //        ("sun.stderr.encoding", "UTF-8"),
        //        ("sun.io.unicode.encoding", "UnicodeBig"),
    ];

    let props_oop = args.get(0).unwrap();
    for (k, v) in props.iter() {
        put_props_kv(jt, props_oop.clone(), k, v);

        if jt.is_meet_ex() {
            unreachable!("jvm_initProperties meet ex");
        }
    }

    //user.dir
    let v = std::env::current_dir().expect("current_dir failed");
    let v = v.to_str().expect("current_dir to_str faield");
    put_props_kv(jt, props_oop.clone(), "user.dir", v);

    //java.io.tmpdir
    let v = std::env::temp_dir();
    let v = v.to_str().expect("temp_dir to_str failed");
    put_props_kv(jt, props_oop.clone(), "java.io.tmpdir", v);

    //user.home
    let v = std::env::home_dir().expect("home_dir failed");
    let v = v.to_str().expect("home_dir to_str failed");
    put_props_kv(jt, props_oop.clone(), "user.home", v);

    //JAVA_HOME
    let v = std::env::var("JAVA_HOME").expect("Please Setup JAVA_HOME env");
    put_props_kv(jt, props_oop.clone(), "java.home", v.as_str());

    //test.src for jdk/test/java/lang/Character/CheckProp.java
    match std::env::var("TEST_SRC") {
        Ok(v) => {
            put_props_kv(jt, props_oop.clone(), "test.src", v.as_str());
        }
        _ => (),
    }

    Ok(Some(props_oop.clone()))
}

fn put_props_kv(jt: &mut JavaThread, props: Oop, k: &str, v: &str) {
    //todo: optimize me
    let cls = {
        let props = util::oop::extract_ref(props.clone());
        let v = props.lock().unwrap();
        match &v.v {
            oop::RefDesc::Inst(inst) => inst.class.clone(),
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

    let k = util::oop::new_java_lang_string2(jt, k);
    let v = util::oop::new_java_lang_string2(jt, v);

    let args = vec![props, k, v];

    let mut jc = JavaCall::new_with_args(jt, mir.clone(), args);
    let mut stack = runtime::Stack::new(1);
    jc.invoke(jt, &mut stack, false);
}

fn jvm_setIn0(_jt: &mut JavaThread, env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    let v = args.get(0).unwrap();
    let cls = { env.lock().unwrap().class.clone() };
    let mut cls = cls.lock().unwrap();
    let id = cls.get_field_id(b"in", b"Ljava/io/InputStream;", true);
    cls.put_static_field_value(id, v.clone());
    Ok(None)
}

fn jvm_setOut0(_jt: &mut JavaThread, env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    let v = args.get(0).unwrap();
    let cls = { env.lock().unwrap().class.clone() };
    let mut cls = cls.lock().unwrap();
    let id = cls.get_field_id(b"out", b"Ljava/io/PrintStream;", true);
    cls.put_static_field_value(id, v.clone());
    Ok(None)
}

fn jvm_setErr0(_jt: &mut JavaThread, env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    let v = args.get(0).unwrap();
    let cls = { env.lock().unwrap().class.clone() };
    let mut cls = cls.lock().unwrap();
    let id = cls.get_field_id(b"err", b"Ljava/io/PrintStream;", true);
    cls.put_static_field_value(id, v.clone());
    Ok(None)
}

fn jvm_mapLibraryName(jt: &mut JavaThread, _env: JNIEnv, args: Vec<Oop>) -> JNIResult {
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

fn jvm_loadLibrary(_jt: &mut JavaThread, _env: JNIEnv, _args: Vec<Oop>) -> JNIResult {
    Ok(None)
}

fn jvm_identityHashCode(jt: &mut JavaThread, env: JNIEnv, args: Vec<Oop>) -> JNIResult {
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

todo optimize: 如何做到不用中转，就达到copy的目的
*/
fn arraycopy_same_obj(
    src: Arc<Mutex<OopRefDesc>>,
    src_pos: usize,
    dest: Arc<Mutex<OopRefDesc>>,
    dest_pos: usize,
    length: usize,
) {
    let is_type_ary = {
        let src = src.lock().unwrap();
        match &src.v {
            oop::RefDesc::TypeArray(_) => true,
            oop::RefDesc::Array(_) => false,
            _ => unreachable!(),
        }
    };

    if is_type_ary {
        let tmp = {
            let src = src.lock().unwrap();

            let mut tmp = Vec::with_capacity(length);

            //just choose the needed region
            match &src.v {
                oop::RefDesc::TypeArray(s) => match s {
                    oop::TypeArrayValue::Char(ary) => {
                        let (_, ary) = ary.split_at(src_pos);
                        ary[..length].iter().for_each(|v| {
                            tmp.push(*v);
                        });
                    }
                    oop::TypeArrayValue::Byte(ary) => {
                        let (_, ary) = ary.split_at(src_pos);
                        ary[..length].iter().for_each(|v| {
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
            oop::RefDesc::TypeArray(ary) => match ary {
                oop::TypeArrayValue::Char(dest) => {
                    let (_, dest) = dest.split_at_mut(dest_pos);
                    dest[..length].copy_from_slice(&tmp[..]);
                }
                oop::TypeArrayValue::Byte(dest) => {
                    let src: Vec<u8> = tmp.iter().map(|v| *v as u8).collect();
                    let (_, dest) = dest.split_at_mut(dest_pos);
                    dest[..length].copy_from_slice(&src[..]);
                }
                t => unreachable!("t = {:?}", t),
            },

            _ => unreachable!(),
        }
    } else {
        let tmp = {
            let ary = src.lock().unwrap();
            match &ary.v {
                oop::RefDesc::Array(ary) => {
                    let mut tmp = Vec::with_capacity(length);
                    for _ in 0..length {
                        tmp.push(oop::consts::get_null());
                    }

                    let (_, ary) = ary.elements.split_at(src_pos);
                    tmp.clone_from_slice(&ary[..length]);
                    tmp
                }
                _ => unreachable!(),
            }
        };

        let mut dest = dest.lock().unwrap();
        match &mut dest.v {
            oop::RefDesc::Array(ary) => {
                let (_, ary) = ary.elements.split_at_mut(dest_pos);
                ary[..length].clone_from_slice(&tmp[..]);
            }
            _ => unreachable!(),
        }
    }
}

fn arraycopy_diff_obj(
    src: Arc<Mutex<OopRefDesc>>,
    src_pos: usize,
    dest: Arc<Mutex<OopRefDesc>>,
    dest_pos: usize,
    length: usize,
) {
    let src = src.lock().unwrap();
    let mut dest = dest.lock().unwrap();

    let is_type_ary = {
        match &src.v {
            oop::RefDesc::TypeArray(_) => true,
            oop::RefDesc::Array(_) => false,
            _ => unreachable!(),
        }
    };

    // error!("src={}, dest={}, length={}, is_type_ary={}", src_pos, dest_pos, length, is_type_ary);

    if is_type_ary {
        match &src.v {
            oop::RefDesc::TypeArray(src_ary) => match src_ary {
                oop::TypeArrayValue::Char(src_ary) => match &mut dest.v {
                    RefDesc::TypeArray(dest_ary) => match dest_ary {
                        oop::TypeArrayValue::Char(dest_ary) => {
                            let (_, dest_ptr) = dest_ary.split_at_mut(dest_pos);
                            let (_, src_ptr) = src_ary.split_at(src_pos);
                            dest_ptr[..length].copy_from_slice(&src_ptr[..length]);
                        }
                        _ => unreachable!(),
                    },
                    _ => unreachable!(),
                },
                oop::TypeArrayValue::Byte(src_ary) => match &mut dest.v {
                    RefDesc::TypeArray(dest_ary) => match dest_ary {
                        oop::TypeArrayValue::Byte(dest_ary) => {
                            let (_, dest_ptr) = dest_ary.split_at_mut(dest_pos);
                            let (_, src_ptr) = src_ary.split_at(src_pos);
                            dest_ptr[..length].copy_from_slice(&src_ptr[..length]);
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
            oop::RefDesc::Array(src) => match &mut dest.v {
                oop::RefDesc::Array(dest) => {
                    let (_, dest_ptr) = dest.elements.split_at_mut(dest_pos);
                    let (_, src_ptr) = src.elements.split_at(src_pos);
                    dest_ptr[..length].clone_from_slice(&src_ptr[..length]);
                }
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }
}

fn jvm_nanoTime(_jt: &mut JavaThread, _env: JNIEnv, _args: Vec<Oop>) -> JNIResult {
    let v = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_nanos(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    };

    Ok(Some(Oop::new_long(v as i64)))
}

fn jvm_currentTimeMillis(_jt: &mut JavaThread, _env: JNIEnv, _args: Vec<Oop>) -> JNIResult {
    let v = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_millis(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    };

    Ok(Some(Oop::new_long(v as i64)))
}
