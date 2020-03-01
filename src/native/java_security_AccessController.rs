#![allow(non_snake_case)]
use crate::classfile;
use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{self, Oop};
use crate::runtime::{self, exception, JavaCall, JavaThread};
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
        new_fn("doPrivileged", "(Ljava/security/PrivilegedExceptionAction;Ljava/security/AccessControlContext;)Ljava/lang/Object;", Box::new(jvm_doPrivileged3)),
    ]
}

fn jvm_doPrivileged(jt: &mut JavaThread, _env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    let v = args.get(0).unwrap();

    let mir = {
        match v {
            Oop::Null => {
                let ex = exception::new(jt, classfile::consts::J_NPE, None);
                return Err(ex);
            }
            Oop::Ref(v) => {
                let v = v.read().unwrap();
                match &v.v {
                    oop::RefKind::Inst(inst) => {
                        let m = {
                            let cls = inst.class.read().unwrap();
                            let id = util::new_method_id(b"run", b"()Ljava/lang/Object;");
                            cls.get_virtual_method(id).unwrap()
                        };

                        m
                    }
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    };

    let args = vec![v.clone()];
    let mut jc = JavaCall::new_with_args(jt, mir, args);
    let area = runtime::DataArea::new(0, 1);
    jc.invoke(jt, Some(&area), false);

    if !jt.is_meet_ex() {
        let mut area = area.borrow_mut();
        let r = area.stack.pop_ref();
        Ok(Some(r))
    } else {
        Ok(None)
    }
}

//todo: re impl
fn jvm_doPrivileged2(jt: &mut JavaThread, env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    jvm_doPrivileged(jt, env, args)
}

//todo: re impl
fn jvm_doPrivileged3(jt: &mut JavaThread, env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    jvm_doPrivileged(jt, env, args)
}

fn jvm_getStackAccessControlContext(
    _jt: &mut JavaThread,
    _env: JNIEnv,
    _args: Vec<Oop>,
) -> JNIResult {
    Ok(Some(oop::consts::get_null()))
}
