use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{Oop, OopRef, OopDesc};
use crate::runtime::{self, JavaThread};
use crate::util;
use std::sync::{Arc, Mutex};
use crate::classfile::types::BytesRef;
use crate::runtime::JavaCall;

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
            Box::new(jvm_initProperties)
        ),
    ]
}

fn jvm_register_natives(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    Ok(None)
}

fn jvm_arraycopy(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    unimplemented!();
    //    Ok(None)
}

fn jvm_initProperties(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    //fixme:
    let props = vec![
        ("java.vm.specification.version", "1.8"),
        ("path.separator", util::PATH_SEP_STR),
        ("file.encoding.pkg", "sun.io"),
        ("os.arch", ""),
        ("os.name", ""),
        ("os.version", ""),
        ("sun.arch.data.model", "64"),
        ("line.separator", "\n"),
        ("file.separator", util::PATH_DELIMITER_STR),
        ("sun.jnu.encoding", "utf8"),
        ("file.encoding", "utf8")
    ];

    let props: Vec<(BytesRef, BytesRef)> = props.iter().map(|(k, v)| {
        (Arc::new(Vec::from(*k)), Arc::new(Vec::from(*v)))
    }).collect();

    match args.get(0) {
        Some(v) => {
            let cls = {
                let v = v.lock().unwrap();
                match &v.v {
                    Oop::Inst(inst) => {
                        inst.class.clone()
                    }
                    _ => unreachable!()
                }
            };

            let mir = {
                let cls = cls.lock().unwrap();
                cls.get_virtual_method(b"(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;", b"put").unwrap()
            };

            let prop = v.clone();
            props.iter().for_each(|(k, v)| {
                let args = vec![
                    prop.clone(),
                    OopDesc::new_str(k.clone()),
                    OopDesc::new_str(v.clone()),
                ];

                let mut jc = JavaCall::new_with_args(jt, mir.clone(), args);
                let mut stack = runtime::Stack::new(0);
                jc.invoke(jt, &mut stack);
            });

            Ok(Some(prop))
        }
        None => unreachable!()
    }
}

