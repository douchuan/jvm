#![allow(non_snake_case)]
use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{self, Oop};
use crate::runtime::{self, exception, thread, JavaCall};
use crate::util;
use classfile::consts as cls_consts;

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
        new_fn("doPrivileged", "(Ljava/security/PrivilegedExceptionAction;Ljava/security/AccessControlContext;)Ljava/lang/Object;", Box::new(jvm_doPrivileged3)),
    ]
}

fn jvm_doPrivileged(_env: JNIEnv, args: &[Oop]) -> JNIResult {
    let v = args.get(0).unwrap();

    let mir = {
        match v {
            Oop::Null => {
                let ex = exception::new(cls_consts::J_NPE, None);
                return Err(ex);
            }
            Oop::Ref(v) => {
                let inst = v.extract_inst();
                let cls = inst.class.get_class();
                cls.get_virtual_method(&util::S_RUN, &util::S_RUN_SIG)
                    .unwrap()
            }
            _ => unreachable!(),
        }
    };

    let args = vec![v.clone()];
    let mut jc = JavaCall::new_with_args(mir, args);
    let area = runtime::DataArea::new(0, 1);
    jc.invoke(Some(&area), false);

    if !thread::is_meet_ex() {
        let mut stack = area.stack.borrow_mut();
        let r = stack.pop_ref();
        Ok(Some(r))
    } else {
        Ok(None)
    }
}

//todo: re impl
fn jvm_doPrivileged2(env: JNIEnv, args: &[Oop]) -> JNIResult {
    jvm_doPrivileged(env, args)
}

//todo: re impl
fn jvm_doPrivileged3(env: JNIEnv, args: &[Oop]) -> JNIResult {
    jvm_doPrivileged(env, args)
}

fn jvm_getStackAccessControlContext(_env: JNIEnv, _args: &[Oop]) -> JNIResult {
    Ok(Some(oop::consts::get_null()))
}
