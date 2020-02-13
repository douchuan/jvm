#![allow(non_snake_case)]

use crate::oop::{ClassRef, OopRef};
use crate::runtime::{Exception, JavaThread};
use crate::util;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

mod java_io_FileDescriptor;
mod java_io_FileInputStream;
mod java_io_FileOutputStream;
mod java_io_UnixFileSystem;
pub mod java_lang_Class;
mod java_lang_ClassLoader;
mod java_lang_Double;
mod java_lang_Float;
mod java_lang_Object;
mod java_lang_String;
mod java_lang_System;
mod java_lang_Thread;
mod java_lang_Throwable;
mod java_security_AccessController;
mod java_util_concurrent_atomic_AtomicLong;
mod sun_misc_Signal;
mod sun_misc_Unsafe;
mod sun_misc_VM;
mod sun_nio_cs_StreamEncoder;
mod sun_reflect_NativeConstructorAccessorImpl;
mod sun_reflect_Reflection;

pub type JNIEnv = Arc<Mutex<Box<JNIEnvStruct>>>;
pub type JNIResult = Result<Option<OopRef>, Exception>;
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

pub fn find_symbol(package: &[u8], name: &[u8], desc: &[u8]) -> Option<JNINativeMethod> {
    let id = vec![package, name, desc].join(util::PATH_DELIMITER);
    let id = String::from_utf8(id).unwrap();
    util::sync_call_ctx(&NATIVES, |h| h.get(&id).map(|it| it.clone()))
}

pub fn init() {
    lazy_static::initialize(&NATIVES);

    let natives = vec![
        (
            "java/io/FileDescriptor",
            java_io_FileDescriptor::get_native_methods(),
        ),
        (
            "java/io/FileInputStream",
            java_io_FileInputStream::get_native_methods(),
        ),
        (
            "java/io/FileOutputStream",
            java_io_FileOutputStream::get_native_methods(),
        ),
        (
            "java/io/UnixFileSystem",
            java_io_UnixFileSystem::get_native_methods(),
        ),
        ("java/lang/Class", java_lang_Class::get_native_methods()),
        (
            "java/lang/ClassLoader",
            java_lang_ClassLoader::get_native_methods(),
        ),
        ("java/lang/Double", java_lang_Double::get_native_methods()),
        ("java/lang/Float", java_lang_Float::get_native_methods()),
        ("java/lang/Object", java_lang_Object::get_native_methods()),
        ("java/lang/String", java_lang_String::get_native_methods()),
        ("java/lang/System", java_lang_System::get_native_methods()),
        ("java/lang/Thread", java_lang_Thread::get_native_methods()),
        (
            "java/lang/Throwable",
            java_lang_Throwable::get_native_methods(),
        ),
        (
            "java/security/AccessController",
            java_security_AccessController::get_native_methods(),
        ),
        (
            "java/util/concurrent/atomic/AtomicLong",
            java_util_concurrent_atomic_AtomicLong::get_native_methods(),
        ),
        ("sun/misc/Signal", sun_misc_Signal::get_native_methods()),
        ("sun/misc/Unsafe", sun_misc_Unsafe::get_native_methods()),
        ("sun/misc/VM", sun_misc_VM::get_native_methods()),
        (
            "sun/nio/cs/StreamEncoder",
            sun_nio_cs_StreamEncoder::get_native_methods(),
        ),
        (
            "sun/reflect/NativeConstructorAccessorImpl",
            sun_reflect_NativeConstructorAccessorImpl::get_native_methods(),
        ),
        (
            "sun/reflect/Reflection",
            sun_reflect_Reflection::get_native_methods(),
        ),
    ];

    util::sync_call_ctx(&NATIVES, |h| {
        natives.iter().for_each(|(package, methods)| {
            methods.iter().for_each(|it| {
                let id =
                    vec![package.as_ref(), it.name, it.signature].join(util::PATH_DELIMITER_STR);

                h.insert(id, it.clone());
            });
        });
    });

    java_lang_Class::init();
}

impl JNINativeMethodStruct {
    pub fn invoke(&self, jt: &mut JavaThread, jni: JNIEnv, args: Vec<OopRef>) -> JNIResult {
        (self.fnptr)(jt, jni, args)
    }
}
