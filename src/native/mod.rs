use crate::oop::{ClassRef, OopRef};
use crate::runtime::JavaThread;
use crate::util;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

mod java_lang_class;
mod java_lang_double;
mod java_lang_float;
mod java_lang_object;
mod java_lang_system;
mod java_lang_thread;
mod java_lang_throwable;
mod java_security_accesscontroller;

pub type JNIEnv = Arc<Mutex<Box<JNIEnvStruct>>>;
pub type JNIResult = Result<Option<OopRef>, Option<OopRef>>;
pub type NativeMethodPtr =
    Box<dyn Fn(&mut JavaThread, JNIEnv, Vec<OopRef>) -> JNIResult + Send + Sync>;
pub type JNINativeMethod = Arc<JNINativeMethodStruct>;

pub struct JNINativeMethodStruct {
    name: &'static str,
    signature: &'static str,
    fnptr: NativeMethodPtr,
}

pub struct JNIEnvStruct {
    //fixme: just for hack，为了跑HelloWorld，暂时放在这里
    pub java_thread_obj: Option<OopRef>,
    pub class: ClassRef,
}

lazy_static! {
    static ref NATIVES: Mutex<HashMap<String, JNINativeMethod>> = {
        let hm = HashMap::new();
        Mutex::new(hm)
    };
}

pub fn new_fn(
    name: &'static str,
    signature: &'static str,
    fnptr: NativeMethodPtr,
) -> JNINativeMethod {
    Arc::new(JNINativeMethodStruct {
        name,
        signature,
        fnptr,
    })
}

pub fn new_jni_env(jt: &mut JavaThread, class: ClassRef) -> JNIEnv {
    Arc::new(Mutex::new(Box::new(JNIEnvStruct {
        java_thread_obj: jt.java_thread_obj.clone(),
        class,
    })))
}

pub fn find_symbol(package: &[u8], desc: &[u8], name: &[u8]) -> Option<JNINativeMethod> {
    let id = vec![package, desc, name].join(util::PATH_DELIMITER);
    let id = String::from_utf8(id).unwrap();
    util::sync_call_ctx(&NATIVES, |h| h.get(&id).map(|it| it.clone()))
}

pub fn init() {
    lazy_static::initialize(&NATIVES);

    let natives = vec![
        ("java/lang/Class", java_lang_class::get_native_methods()),
        ("java/lang/Double", java_lang_double::get_native_methods()),
        ("java/lang/Float", java_lang_float::get_native_methods()),
        ("java/lang/Object", java_lang_object::get_native_methods()),
        ("java/lang/System", java_lang_system::get_native_methods()),
        ("java/lang/Thread", java_lang_thread::get_native_methods()),
        (
            "java/lang/Throwable",
            java_lang_throwable::get_native_methods(),
        ),
        (
            "java/security/AccessController",
            java_security_accesscontroller::get_native_methods(),
        ),
    ];

    util::sync_call_ctx(&NATIVES, |h| {
        natives.iter().for_each(|(package, methods)| {
            methods.iter().for_each(|it| {
                let id =
                    vec![package.as_ref(), it.signature, it.name].join(util::PATH_DELIMITER_STR);

                h.insert(id, it.clone());
            });
        });
    });

    java_lang_class::init();
}

impl JNINativeMethodStruct {
    pub fn invoke(&self, jt: &mut JavaThread, jni: JNIEnv, args: Vec<OopRef>) -> JNIResult {
        (self.fnptr)(jt, jni, args)
    }
}
