#![allow(non_snake_case)]

use crate::native::{self, new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{self, Oop, OopRef, RefKind};
use crate::runtime::{self, JavaCall};
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
        new_fn("currentTimeMillis", "()J", Box::new(jvm_currentTimeMillis)),
        //Note: just for debug
        //        new_fn("getProperty", "(Ljava/lang/String;)Ljava/lang/String;", Box::new(jvm_getProperty)),
    ]
}

fn jvm_registerNatives(_env: JNIEnv, _args: Vec<Oop>) -> JNIResult {
    Ok(None)
}

fn jvm_arraycopy(_env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    let src = args.get(0).unwrap();
    let src_pos = args.get(1).unwrap().extract_int();
    let dest = args.get(2).unwrap();
    let dest_pos = args.get(3).unwrap().extract_int();
    let length = args.get(4).unwrap().extract_int();

    //todo: do check & throw exception

    if length == 0 {
        return Ok(None);
    }

    let is_same_obj = OopRef::is_eq(src.extract_ref(), dest.extract_ref());

    if is_same_obj {
        arraycopy_same_obj(
            src.extract_ref(),
            src_pos as usize,
            dest_pos as usize,
            length as usize,
        );
    } else {
        arraycopy_diff_obj(
            src.extract_ref(),
            src_pos as usize,
            dest.extract_ref(),
            dest_pos as usize,
            length as usize,
        );
    }

    Ok(None)
}

fn jvm_initProperties(_env: JNIEnv, args: Vec<Oop>) -> JNIResult {
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
        ("java.specification.vendor", "Oracle Corporation"),
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
        ("sun.stdout.encoding", "UTF-8"),
        ("sun.stderr.encoding", "UTF-8"),
        ("user.language", "en"),
        ("user.name", "chuan"),
        ("user.region", "US"),
        //        ("java.security.manager", ""),
        //        ("sun.jnu.encoding", "UTF-8"),
        //        ("sun.io.unicode.encoding", "UnicodeBig"),
    ];

    let props_oop = args.get(0).unwrap();
    for (k, v) in props.iter() {
        put_props_kv(props_oop, k, v);
    }

    //user.dir
    let v = std::env::current_dir().expect("current_dir failed");
    let v = v.to_str().expect("current_dir to_str faield");
    put_props_kv(props_oop, "user.dir", v);

    //java.io.tmpdir
    let v = std::env::temp_dir();
    let v = v.to_str().expect("temp_dir to_str failed");
    put_props_kv(props_oop, "java.io.tmpdir", v);

    //user.home
    let v = dirs::home_dir().expect("get home_dir failed");
    let v = v.to_str().expect("home_dir to_str failed");
    put_props_kv(props_oop, "user.home", v);

    //JAVA_HOME
    let v = std::env::var("JAVA_HOME").expect("Please Setup JAVA_HOME env");
    put_props_kv(props_oop, "java.home", v.as_str());

    //test.src for jdk/test/java/lang/Character/CheckProp.java
    match std::env::var("TEST_SRC") {
        Ok(v) => {
            put_props_kv(props_oop, "test.src", v.as_str());
        }
        _ => (),
    }

    let jt = runtime::thread::current_java_thread();
    if jt.read().unwrap().is_meet_ex() {
        unreachable!("jvm_initProperties meet ex");
    }

    Ok(Some(props_oop.clone()))
}

fn put_props_kv(props: &Oop, k: &str, v: &str) {
    //todo: optimize me
    let cls = {
        let rf = props.extract_ref();
        let inst = rf.extract_inst();
        inst.class.clone()
    };

    let mir = {
        let cls = cls.read().unwrap();
        cls.get_virtual_method(
            b"put",
            b"(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
        )
        .unwrap()
    };

    let k = util::oop::new_java_lang_string2(k);
    let v = util::oop::new_java_lang_string2(v);

    let args = vec![props.clone(), k, v];

    let mut jc = JavaCall::new_with_args(mir.clone(), args);
    let area = runtime::DataArea::new(0, 1);
    jc.invoke(Some(area), false);
}

fn jvm_setIn0(env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    let v = args.get(0).unwrap();
    let cls = env.read().unwrap().class.clone();
    let mut cls = cls.write().unwrap();
    let id = cls.get_field_id(b"in", b"Ljava/io/InputStream;", true);
    cls.put_static_field_value(id, v.clone());
    Ok(None)
}

fn jvm_setOut0(env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    let v = args.get(0).unwrap();
    let cls = env.read().unwrap().class.clone();
    let mut cls = cls.write().unwrap();
    let id = cls.get_field_id(b"out", b"Ljava/io/PrintStream;", true);
    cls.put_static_field_value(id, v.clone());
    Ok(None)
}

fn jvm_setErr0(env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    let v = args.get(0).unwrap();
    let cls = env.read().unwrap().class.clone();
    let mut cls = cls.write().unwrap();
    let id = cls.get_field_id(b"err", b"Ljava/io/PrintStream;", true);
    cls.put_static_field_value(id, v.clone());
    Ok(None)
}

fn jvm_mapLibraryName(_env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    let v = args.get(0).unwrap();
    let s = OopRef::java_lang_string(v.extract_ref());

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

    let v = util::oop::new_java_lang_string2(&name);

    Ok(Some(v))
}

fn jvm_loadLibrary(_env: JNIEnv, _args: Vec<Oop>) -> JNIResult {
    Ok(None)
}

fn jvm_identityHashCode(env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    native::java_lang_Object::jvm_hashCode(env, args)
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

fn arraycopy_same_obj(buf: Arc<OopRef>, src_pos: usize, dest_pos: usize, length: usize) {
    let is_type_ary = {
        let ptr = buf.get_raw_ptr();
        unsafe {
            match &(*ptr).v {
                oop::RefKind::TypeArray(_) => true,
                oop::RefKind::Array(_) => false,
                _ => unreachable!(),
            }
        }
    };

    if is_type_ary {
        let ptr = buf.get_mut_raw_ptr();
        unsafe {
            match &mut (*ptr).v {
                oop::RefKind::TypeArray(ary) => match ary {
                    oop::TypeArrayDesc::Char(ary) => {
                        ary.copy_within(src_pos..(src_pos + length), dest_pos)
                    }
                    oop::TypeArrayDesc::Byte(ary) => {
                        ary.copy_within(src_pos..(src_pos + length), dest_pos)
                    }
                    t => unreachable!("t = {:?}", t),
                },

                _ => unreachable!(),
            }
        }
    } else {
        /*
        todo optimize: How to achieve the purpose of copy without tmp
        */
        let tmp = {
            let ary = buf.extract_array();
            let mut tmp = Vec::with_capacity(length);
            tmp.resize(length, oop::consts::get_null());

            let (_, ary) = ary.elements.split_at(src_pos);
            tmp.clone_from_slice(&ary[..length]);
            tmp
        };

        let ary = buf.extract_mut_array();
        let (_, ary) = ary.elements.split_at_mut(dest_pos);
        ary[..length].clone_from_slice(&tmp[..]);
    }
}

fn arraycopy_diff_obj(
    src: Arc<OopRef>,
    src_pos: usize,
    dest: Arc<OopRef>,
    dest_pos: usize,
    length: usize,
) {
    let is_type_ary = {
        let ptr = src.get_raw_ptr();
        unsafe {
            match &(*ptr).v {
                oop::RefKind::TypeArray(_) => true,
                oop::RefKind::Array(_) => false,
                _ => unreachable!(),
            }
        }
    };

    // error!("src={}, dest={}, length={}, is_type_ary={}", src_pos, dest_pos, length, is_type_ary);

    let src_ptr = src.get_raw_ptr();
    let dst_ptr = dest.get_mut_raw_ptr();

    if is_type_ary {
        unsafe {
            match &(*src_ptr).v {
                oop::RefKind::TypeArray(src_ary) => match src_ary {
                    oop::TypeArrayDesc::Char(src_ary) => match &mut (*dst_ptr).v {
                        RefKind::TypeArray(dest_ary) => match dest_ary {
                            oop::TypeArrayDesc::Char(dest_ary) => {
                                let (_, dest_ptr) = dest_ary.split_at_mut(dest_pos);
                                let (_, src_ptr) = src_ary.split_at(src_pos);
                                dest_ptr[..length].copy_from_slice(&src_ptr[..length]);
                            }
                            _ => unreachable!(),
                        },
                        _ => unreachable!(),
                    },
                    oop::TypeArrayDesc::Byte(src_ary) => match &mut (*dst_ptr).v {
                        RefKind::TypeArray(dest_ary) => match dest_ary {
                            oop::TypeArrayDesc::Byte(dest_ary) => {
                                let (_, dest_ptr) = dest_ary.split_at_mut(dest_pos);
                                let (_, src_ptr) = src_ary.split_at(src_pos);
                                dest_ptr[..length].copy_from_slice(&src_ptr[..length]);
                            }
                            _ => unreachable!(),
                        },
                        _ => unreachable!(),
                    },
                    oop::TypeArrayDesc::Int(src_ary) => match &mut (*dst_ptr).v {
                        RefKind::TypeArray(dest_ary) => match dest_ary {
                            oop::TypeArrayDesc::Int(dest_ary) => {
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
        }
    } else {
        unsafe {
            match &(*src_ptr).v {
                oop::RefKind::Array(src) => match &mut (*dst_ptr).v {
                    oop::RefKind::Array(dest) => {
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
}

fn jvm_nanoTime(_env: JNIEnv, _args: Vec<Oop>) -> JNIResult {
    let v = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_nanos(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    };

    Ok(Some(Oop::new_long(v as i64)))
}

fn jvm_currentTimeMillis(_env: JNIEnv, _args: Vec<Oop>) -> JNIResult {
    let v = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_millis(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    };

    Ok(Some(Oop::new_long(v as i64)))
}
