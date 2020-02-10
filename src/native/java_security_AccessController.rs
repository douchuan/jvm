#![allow(non_snake_case)]
use crate::classfile;
use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{self, Oop, OopRef};
use crate::runtime::{Exception, JavaCall, JavaThread, Stack};
use crate::util;
use std::sync::{Arc, Mutex};

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![
        new_fn(
            "doPrivileged",
            "(Ljava/security/PrivilegedAction;)Ljava/lang/Object;",
            Box::new(jvm_doPrivileged),
        ),
        new_fn(
            "doPrivileged",
            "(Ljava/security/PrivilegedExceptionAction;)Ljava/lang/Object;",
            Box::new(jvm_doPrivileged2),
        ),
        new_fn(
            "getStackAccessControlContext",
            "()Ljava/security/AccessControlContext;",
            Box::new(jvm_getStackAccessControlContext),
        ),
    ]
}

fn jvm_doPrivileged(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    match args.get(0) {
        Some(v) => {
            let mut mir = None;

            {
                let v = v.lock().unwrap();
                match &v.v {
                    Oop::Null => {
                        let cls_name = Vec::from(classfile::consts::J_NPE);
                        let exception = Exception {
                            cls_name: new_ref!(cls_name),
                            msg: None,
                            ex_oop: None,
                        };
                        return Err(exception);
                    }
                    Oop::Inst(inst) => {
                        let m = {
                            let cls = inst.class.lock().unwrap();
                            let id = util::new_method_id(b"run", b"()Ljava/lang/Object;");
                            cls.get_virtual_method(id).unwrap()
                        };
                        mir = Some(m);
                    }
                    _ => unreachable!(),
                }
            }

            let args = vec![v.clone()];
            let mut jc = JavaCall::new_with_args(jt, mir.unwrap(), args);
            let mut stack = Stack::new(1);
            jc.invoke(jt, &mut stack, false);

            if !jt.is_meet_ex() {
                let r = stack.pop_ref();
                Ok(Some(r))
            } else {
                Ok(None)
            }
        }
        None => unreachable!(),
    }
}

fn jvm_doPrivileged2(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    jvm_doPrivileged(jt, env, args)
}

fn jvm_getStackAccessControlContext(
    jt: &mut JavaThread,
    env: JNIEnv,
    args: Vec<OopRef>,
) -> JNIResult {
    Ok(Some(oop::consts::get_null()))
}
