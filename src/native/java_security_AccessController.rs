#![allow(non_snake_case)]
use crate::classfile;
use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{self, Oop};
use crate::runtime::{exception, JavaCall, JavaThread, Stack};
use crate::types::OopRef;
use crate::util;

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

fn jvm_doPrivileged(jt: &mut JavaThread, _env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let v = args.get(0).unwrap();

    let mir = {
        let v = v.lock().unwrap();
        match &v.v {
            Oop::Null => {
                let ex = exception::new(jt, classfile::consts::J_NPE, None);
                return Err(ex);
            }
            Oop::Inst(inst) => {
                let m = {
                    let cls = inst.class.lock().unwrap();
                    let id = util::new_method_id(b"run", b"()Ljava/lang/Object;");
                    cls.get_virtual_method(id).unwrap()
                };

                m
            }
            _ => unreachable!(),
        }
    };

    let args = vec![v.clone()];
    let mut jc = JavaCall::new_with_args(jt, mir, args);
    let mut stack = Stack::new(1);
    jc.invoke(jt, &mut stack, false);

    if !jt.is_meet_ex() {
        let r = stack.pop_ref();
        Ok(Some(r))
    } else {
        Ok(None)
    }
}

fn jvm_doPrivileged2(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    jvm_doPrivileged(jt, env, args)
}

fn jvm_getStackAccessControlContext(
    _jt: &mut JavaThread,
    _env: JNIEnv,
    _args: Vec<OopRef>,
) -> JNIResult {
    Ok(Some(oop::consts::get_null()))
}
