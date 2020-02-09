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
        new_fn("registerNatives", "()V", Box::new(jvm_register_natives)),
        new_fn(
            "initProperties",
            "(Ljava/util/Properties;)Ljava/util/Properties;",
            Box::new(jvm_initProperties),
        ),
    ]
}

fn jvm_register_natives(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    Ok(None)
}

fn jvm_arraycopy(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {

    let src = {
        match args.get(0) {
            Some(v) => v.clone(),
            _ => unreachable!()
        }
    };
    let src_pos = {
        let arg1 = match args.get(1) {
            Some(v) => v.clone(),
            _ => unreachable!()
        };

        let arg1 = arg1.lock().unwrap();
        match &arg1.v {
            Oop::Int(v) => *v,
            _ => unreachable!()
        }
    };
    let dest= {
        match args.get(2) {
            Some(v) => v.clone(),
            _ => unreachable!()
        }
    };
    let dest_pos = {
        let arg3 = match args.get(3) {
            Some(v) => v.clone(),
            _ => unreachable!()
        };

        let arg3 = arg3.lock().unwrap();
        match &arg3.v {
            Oop::Int(v) => *v,
            _ => unreachable!()
        }
    };
    let length= {
        let arg4 = match args.get(4) {
            Some(v) => v.clone(),
            _ => unreachable!()
        };

        let arg4 = arg4.lock().unwrap();
        match &arg4.v {
            Oop::Int(v) => *v,
            _ => unreachable!()
        }
    };

    //todo: do check

    if length == 0 {
        return Ok(None);
    }

    let is_str = {
       let src = src.lock().unwrap();
        match &src.v {
            Oop::Array(ary) => false,
            Oop::Str(_) => true,
            _ => unreachable!()
        }
    };

    if is_str {
        let src= {
            let ary = src.lock().unwrap();
            match &ary.v {
                Oop::Str(s) => s.clone(),
                _ => unreachable!()
            }
        };

        //just construct the needed region
        let src: Vec<OopRef> = src[src_pos as usize..(src_pos + length - 1) as usize]
            .iter()
            .map(|v| {
                OopDesc::new_int(*v as i32)
            }).collect();

        let (dest_cls, mut dest) = {
            let ary = dest.lock().unwrap();
            match &ary.v {
                Oop::Array(ary) => (ary.class.clone(), ary.elements.clone()),
                _ => unreachable!()
            }
        };

        dest[dest_pos as usize..(dest_pos + length - 1) as usize].clone_from_slice(&src[..]);

        let oop = OopDesc::new_ary2(dest_cls, dest);
        Ok(Some(oop))
    } else {
        let src= {
            let ary = src.lock().unwrap();
            match &ary.v {
                Oop::Array(ary) => ary.elements.clone(),
                _ => unreachable!()
            }
        };
        let (dest_cls,  mut dest)= {
            let ary = dest.lock().unwrap();
            match &ary.v {
                Oop::Array(ary) => (ary.class.clone(), ary.elements.clone()),
                _ => unreachable!()
            }
        };

        dest[dest_pos as usize..(dest_pos + length - 1) as usize].clone_from_slice(&src[src_pos as usize..(src_pos + length - 1) as usize]);

        let oop = OopDesc::new_ary2(dest_cls, dest);

        Ok(Some(oop))
    }
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
