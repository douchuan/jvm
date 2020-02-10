#![allow(non_snake_case)]

use crate::classfile::types::BytesRef;
use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{Oop, OopDesc, OopRef};
use crate::runtime::JavaCall;
use crate::runtime::{self, JavaThread};
use crate::util;
use std::sync::{Arc, Mutex};

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
    ]
}

fn jvm_registerNatives(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    Ok(None)
}

fn jvm_arraycopy(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
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

    let is_str = util::oop::is_str(src.clone());
    if is_str {
        let src: Vec<OopRef> = {
            let src = src.lock().unwrap();
            match &src.v {
                Oop::Str(s) => {
                    //just construct the needed region
                    s[src_pos as usize..(src_pos + length - 1) as usize]
                        .iter()
                        .map(|v| OopDesc::new_int(*v as i32))
                        .collect()
                }
                _ => unreachable!(),
            }
        };

        let mut dest = dest.lock().unwrap();
        match &mut dest.v {
            Oop::Array(dest) => {
                dest.elements[dest_pos as usize..(dest_pos + length - 1) as usize]
                    .clone_from_slice(&src[..]);
            }
            _ => unreachable!(),
        }
    } else {
        //这里为了避免clone一个大数组，做了优化
        //src 和 dest 有可能是同一个对象，所以，两个不能同时上锁
        if Arc::ptr_eq(&src, &dest) {
            let src = {
                let ary = src.lock().unwrap();
                match &ary.v {
                    Oop::Array(ary) => {
                        let mut new_ary = Vec::new();
                        new_ary.clone_from_slice(
                            &ary.elements[src_pos as usize..(src_pos + length - 1) as usize],
                        );
                        new_ary
                    }
                    _ => unreachable!(),
                }
            };

            let mut dest = dest.lock().unwrap();
            match &mut dest.v {
                Oop::Array(ary) => {
                    ary.elements[dest_pos as usize..(dest_pos + length - 1) as usize]
                        .clone_from_slice(&src[..]);
                }
                _ => unreachable!(),
            }
        } else {
            let src = src.lock().unwrap();
            match &src.v {
                Oop::Array(src) => {
                    let mut dest = dest.lock().unwrap();
                    match &mut dest.v {
                        Oop::Array(dest) => {
                            dest.elements[dest_pos as usize..(dest_pos + length - 1) as usize]
                                .clone_from_slice(
                                    &src.elements
                                        [src_pos as usize..(src_pos + length - 1) as usize],
                                );
                        }
                        _ => unreachable!(),
                    }
                }
                _ => unreachable!(),
            }
        }
    }

    Ok(None)
}

fn jvm_initProperties(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    //fixme:
    let props = vec![
        ("java.vm.specification.version", "1.8"),
        ("path.separator", util::PATH_SEP_STR),
        ("file.encoding.pkg", "sun.io"),
        ("os.arch", "xxx"),
        ("os.name", "xxx"),
        ("os.version", "xxx"),
        ("sun.arch.data.model", "64"),
        ("line.separator", "\n"),
        ("file.separator", util::PATH_DELIMITER_STR),
        ("sun.jnu.encoding", "utf8"),
        ("file.encoding", "utf8"),
    ];

    let props: Vec<(BytesRef, BytesRef)> = props
        .iter()
        .map(|(k, v)| {
            let k = Vec::from(*k);
            let k = new_ref!(k);
            let v = Vec::from(*v);
            let v = new_ref!(v);
            (k, v)
        })
        .collect();

    match args.get(0) {
        Some(v) => {
            let cls = {
                let v = v.lock().unwrap();
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

            let prop = v.clone();
            for it in props.iter() {
                let args = vec![
                    prop.clone(),
                    OopDesc::new_str(it.0.clone()),
                    OopDesc::new_str(it.1.clone()),
                ];

                let mut jc = JavaCall::new_with_args(jt, mir.clone(), args);
                let mut stack = runtime::Stack::new(1);
                jc.invoke(jt, &mut stack, false);

                //fixme: should be removed
                if jt.is_meet_ex() {
                    error!("jvm_initProperties meet ex");
                    break;
                }
            }

            Ok(Some(prop))
        }
        None => unreachable!(),
    }
}

fn jvm_setIn0(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let v = args.get(0).unwrap();
    let cls = { env.lock().unwrap().class.clone() };
    let mut cls = cls.lock().unwrap();
    let id = cls.get_field_id(b"in", b"Ljava/io/InputStream;", true);
    cls.put_static_field_value(id, v.clone());
    Ok(None)
}
