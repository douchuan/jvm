use crate::oop::OopRef;
use crate::util;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

mod system;

pub type JNIEnv = Arc<Mutex<Box<JNIEnvStruct>>>;
pub type NativeMethodPtr = Box<dyn Fn(JNIEnv, Vec<OopRef>) -> Option<OopRef> + Send + Sync>;
pub type JNINativeMethod = Arc<JNINativeMethodStruct>;

pub struct JNINativeMethodStruct {
    name: &'static [u8],
    signature: &'static [u8],
    fnptr: NativeMethodPtr,
}

pub struct JNIEnvStruct {

}

lazy_static! {
    static ref NATIVES: Mutex<HashMap<String, JNINativeMethod>> = {
        let hm = HashMap::new();
        Mutex::new(hm)
    };
}

pub fn new_fn(name: &'static [u8], signature: &'static [u8], fnptr: NativeMethodPtr) -> JNINativeMethod {
    Arc::new(JNINativeMethodStruct {
        name,
        signature,
        fnptr
    })
}

pub fn new_jni_env() -> JNIEnv {
    Arc::new(Mutex::new(Box::new(JNIEnvStruct {

    })))
}

pub fn find_symbol(package: &[u8], desc: &[u8], name: &[u8]) -> Option<JNINativeMethod> {
    let id = vec![package, desc, name].join(util::PATH_DELIMITER);
    let id = String::from_utf8(id).unwrap();
    util::sync_call_ctx(&NATIVES, |h| {
        h.get(&id).map(|it| it.clone())
    })
}

pub fn init() {
    lazy_static::initialize(&NATIVES);

    let natives = vec! [
        (b"java/lang/System", system::get_native_methods()),
    ];

    util::sync_call_ctx(&NATIVES, |h| {
        natives.iter().for_each(|(package, methods)| {
            methods.iter().for_each(|it| {
                let id = vec![
                    package.as_ref(),
                    it.signature,
                    it.name
                ].join(util::PATH_DELIMITER);
                let id = String::from_utf8(id).unwrap();

                h.insert(id, it.clone());
            });
        });
    });
}

impl JNINativeMethodStruct {
    pub fn invoke(&self, jni: JNIEnv, args: Vec<OopRef>) -> Option<OopRef> {
        (self.fnptr)(jni, args)
    }
}

