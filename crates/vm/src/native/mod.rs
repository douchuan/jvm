#![allow(non_snake_case)]

use crate::oop::Oop;
use crate::types::ClassRef;
use rustc_hash::FxHashMap;
use std::sync::{Arc, RwLock};

mod common;

mod java_io_FileDescriptor;
mod java_io_FileInputStream;
mod java_io_FileOutputStream;
mod java_io_UnixFileSystem;
pub mod java_lang_Class;
mod java_lang_ClassLoader;
mod java_lang_Double;
mod java_lang_Float;
mod java_lang_Object;
mod java_lang_Runtime;
mod java_lang_String;
mod java_lang_System;
mod java_lang_Thread;
mod java_lang_Throwable;
mod java_lang_reflect_Array;
mod java_lang_reflect_Proxy;
mod java_security_AccessController;
mod java_util_concurrent_atomic_AtomicLong;
mod sun_misc_Signal;
mod sun_misc_URLClassPath;
mod sun_misc_Unsafe;
mod sun_misc_VM;
mod sun_nio_cs_StreamEncoder;
mod sun_reflect_ConstantPool;
mod sun_reflect_NativeConstructorAccessorImpl;
mod sun_reflect_NativeMethodAccessorImpl;
mod sun_reflect_Reflection;

pub type JNIEnv = Arc<RwLock<Box<JNIEnvStruct>>>;
pub type JNIResult = Result<Option<Oop>, Oop>;
pub type NativeMethodPtr = Box<dyn Fn(JNIEnv, &[Oop]) -> JNIResult + Send + Sync>;
pub type JNINativeMethod = Arc<JNINativeMethodStruct>;

pub struct JNINativeMethodStruct {
    name: &'static str,
    signature: &'static str,
    fnptr: NativeMethodPtr,
}

pub struct JNIEnvStruct {
    pub class: ClassRef,
}

lazy_static! {
    //(class name, method name, method signature) -> JNINativeMethod
    static ref NATIVES: FxHashMap<(&'static str, &'static str, &'static str), JNINativeMethod> = {
        create_native_fn_tables()
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

pub fn new_jni_env(class: ClassRef) -> JNIEnv {
    Arc::new(RwLock::new(Box::new(JNIEnvStruct { class })))
}

pub fn find_symbol(package: &[u8], name: &[u8], desc: &[u8]) -> Option<JNINativeMethod> {
    let package = unsafe { std::str::from_utf8_unchecked(package) };
    let name = unsafe { std::str::from_utf8_unchecked(name) };
    let desc = unsafe { std::str::from_utf8_unchecked(desc) };

    let k = (package, name, desc);
    NATIVES.get(&k).cloned()
}

pub fn init() {
    lazy_static::initialize(&NATIVES);
    java_lang_Class::init();
}

impl JNINativeMethodStruct {
    pub fn invoke(&self, jni: JNIEnv, args: &[Oop]) -> JNIResult {
        (self.fnptr)(jni, args)
    }
}

fn create_native_fn_tables(
) -> FxHashMap<(&'static str, &'static str, &'static str), JNINativeMethod> {
    let mut dict = FxHashMap::default();
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
        (
            "java/lang/reflect/Array",
            java_lang_reflect_Array::get_native_methods(),
        ),
        (
            "java/lang/reflect/Proxy",
            java_lang_reflect_Proxy::get_native_methods(),
        ),
        ("java/lang/Runtime", java_lang_Runtime::get_native_methods()),
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
        (
            "sun/misc/URLClassPath",
            sun_misc_URLClassPath::get_native_methods(),
        ),
        ("sun/misc/VM", sun_misc_VM::get_native_methods()),
        (
            "sun/nio/cs/StreamEncoder",
            sun_nio_cs_StreamEncoder::get_native_methods(),
        ),
        (
            "sun/reflect/ConstantPool",
            sun_reflect_ConstantPool::get_native_methods(),
        ),
        (
            "sun/reflect/NativeConstructorAccessorImpl",
            sun_reflect_NativeConstructorAccessorImpl::get_native_methods(),
        ),
        (
            "sun/reflect/NativeMethodAccessorImpl",
            sun_reflect_NativeMethodAccessorImpl::get_native_methods(),
        ),
        (
            "sun/reflect/Reflection",
            sun_reflect_Reflection::get_native_methods(),
        ),
    ];

    {
        natives.iter().for_each(|(package, methods)| {
            methods.iter().for_each(|it| {
                let k = (*package, it.name, it.signature);
                dict.insert(k, it.clone());
            });
        });
    }

    dict
}
