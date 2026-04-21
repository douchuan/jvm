use crate::oop;
use crate::oop::{Class, Oop};
use crate::runtime::{self, require_class3};
use crate::types::JavaThreadRef;
use crate::util;
use crate::{native, new_br};
use classfile::consts::{
    J_ARRAY_INDEX_OUT_OF_BOUNDS, J_CLASS, J_CLASS_NOT_FOUND, J_CLONEABLE, J_FIELD, J_INPUT_STREAM,
    J_INTERNAL_ERROR, J_IOEXCEPTION, J_METHOD_CTOR, J_NPE, J_OBJECT, J_PRINT_STREAM,
    J_SECURITY_MANAGER, J_SERIALIZABLE, J_STRING, J_SYSTEM, J_THREAD, J_THREAD_GROUP, J_THROWABLE,
};
use std::borrow::BorrowMut;
use std::sync::Arc;

pub fn initialize_jvm() {
    initialize_vm_structs();

    let thread_cls = oop::class::load_and_init(J_THREAD);

    // Create a minimal Thread instance
    let init_thread_oop = oop::Oop::new_inst(thread_cls.clone());

    let jt = runtime::thread::current_java_thread();
    jt.write()
        .unwrap()
        .set_java_thread_obj(init_thread_oop.clone());

    let _ = oop::class::load_and_init(J_INPUT_STREAM);
    let _ = oop::class::load_and_init(J_PRINT_STREAM);

    hack_classes();

    // Try System.initializeSystemClass() if it exists (JDK 8)
    // In JDK 9+ this method doesn't exist and clinit handles initialization
    if let Some(cls) = require_class3(None, J_SYSTEM) {
        if let Ok(method) = cls.get_class().get_static_method(&new_br("initializeSystemClass"), &new_br("()V")) {
            let mut jc = runtime::invoke::JavaCall::new_with_args(method, vec![]);
            jc.invoke(None, false);
        }
    }

    // JDK 9+: System.out/in/err are not initialized by clinit alone.
    // Manually create PrintStream instances and set them as static fields.
    setup_system_streams();

    //setup security (best effort)
    let _ = oop::class::load_and_init(b"sun/security/provider/Sun");
    let _ = oop::class::load_and_init(b"sun/security/rsa/SunRsaSign");
}

fn initialize_vm_structs() {
    let class_obj = oop::class::load_and_init(J_CLASS);
    native::java_lang_Class::create_delayed_mirrors();
    native::java_lang_Class::create_delayed_ary_mirrors();

    let _ = oop::class::load_and_init(J_OBJECT);
    let string_cls = oop::class::load_and_init(J_STRING);
    {
        let cls = string_cls.get_class();
        // JDK 9+: value is byte[] ([B); pre-JDK 9: char[] ([C)
        let fir = cls.get_field_id(&new_br("value"), &new_br("[B"), false);
        util::oop::set_java_lang_string_value_offset(fir.offset);
    }

    let integer_cls = oop::class::load_and_init(b"java/lang/Integer");
    {
        let cls = integer_cls.get_class();
        let fir = cls.get_field_id(&new_br("value"), &new_br("I"), false);
        util::oop::set_java_lang_integer_value_offset(fir.offset);
    }

    let _ = oop::class::load_and_init(J_CLONEABLE);
    let _ = oop::class::load_and_init(J_SERIALIZABLE);
    let _ = oop::class::load_and_init(J_NPE);
    let _ = oop::class::load_and_init(J_ARRAY_INDEX_OUT_OF_BOUNDS);
    let _ = oop::class::load_and_init(J_CLASS_NOT_FOUND);
    let _ = oop::class::load_and_init(J_INTERNAL_ERROR);
    let _ = oop::class::load_and_init(J_IOEXCEPTION);
    let _ = oop::class::load_and_init(J_FIELD);
    let _ = oop::class::load_and_init(J_METHOD_CTOR);
    let _ = oop::class::load_and_init(J_THROWABLE);
}

fn hack_classes() {
    let charset_cls = oop::class::load_and_init(b"java/nio/charset/Charset");
    let ascii_charset_cls = oop::class::load_and_init(b"sun/nio/cs/US_ASCII");

    let ascii_inst = oop::Oop::new_inst(ascii_charset_cls.clone());
    let args = vec![ascii_inst.clone()];
    runtime::invoke::invoke_ctor(ascii_charset_cls, new_br("()V"), args);

    {
        let cls = charset_cls.get_class();
        let id = cls.get_field_id(
            &new_br("defaultCharset"),
            &new_br("Ljava/nio/charset/Charset;"),
            true,
        );
        cls.put_static_field_value(id, ascii_inst);
    }

    let encoder = oop::class::load_and_init(b"sun/nio/cs/StreamEncoder");
    {
        let cls = encoder.get_class();
        cls.hack_as_native(b"forOutputStreamWriter", b"(Ljava/io/OutputStream;Ljava/lang/Object;Ljava/lang/String;)Lsun/nio/cs/StreamEncoder;");
    }

    let system = oop::class::load_and_init(b"java/lang/System");

    {
        let cls = system.get_class();
        cls.hack_as_native(b"load", b"(Ljava/lang/String;)V");
        cls.hack_as_native(b"loadLibrary", b"(Ljava/lang/String;)V");
    }

    // JDK 9+: fillInStackTrace() delegates to fillInStackTrace(int) internally.
    // Make () variant native to bypass JDK bytecode that causes NPE cascade.
    let throwable = oop::class::load_and_init(b"java/lang/Throwable");
    {
        let cls = throwable.get_class();
        cls.hack_as_native(b"fillInStackTrace", b"()Ljava/lang/Throwable;");
    }

    // JDK 9+: dispatchUncaughtException calls threadState/isTerminated which aren't implemented.
    let thread_cls = oop::class::load_and_init(b"java/lang/Thread");
    {
        let cls = thread_cls.get_class();
        cls.hack_as_native(b"dispatchUncaughtException", b"(Ljava/lang/Throwable;)V");
    }
}

/// Manually create a dummy System.out PrintStream and hack PrintStream methods as native.
/// PrintStream's Java bytecode accesses fields in FilterOutputStream which aren't set up.
/// By making println/writeln native, we bypass the bytecode entirely.
fn setup_system_streams() {
    // Load PrintStream without initializing (avoids clinit which needs IO infrastructure)
    let ps_cls = runtime::require_class3(None, b"java/io/PrintStream").unwrap();

    // Hack PrintStream methods as native to bypass bytecode that accesses FilterOutputStream fields
    {
        let cls = ps_cls.get_class();
        // println variants
        cls.hack_as_native(b"println", b"(I)V");
        cls.hack_as_native(b"println", b"(J)V");
        cls.hack_as_native(b"println", b"(F)V");
        cls.hack_as_native(b"println", b"(D)V");
        cls.hack_as_native(b"println", b"(Z)V");
        cls.hack_as_native(b"println", b"(C)V");
        cls.hack_as_native(b"println", b"(Ljava/lang/String;)V");
        cls.hack_as_native(b"println", b"(Ljava/lang/Object;)V");
        cls.hack_as_native(b"println", b"()V");
        // print variants
        cls.hack_as_native(b"print", b"(I)V");
        cls.hack_as_native(b"print", b"(Ljava/lang/String;)V");
        cls.hack_as_native(b"print", b"(Ljava/lang/Object;)V");
        // Internal methods that access out field
        cls.hack_as_native(b"write", b"(I)V");
        cls.hack_as_native(b"write", b"([B)V");
        cls.hack_as_native(b"write", b"([BII)V");
        cls.hack_as_native(b"flush", b"()V");
        cls.hack_as_native(b"close", b"()V");
    }

    // Create instance (clinit won't run until first proper use)
    let ps_out = Oop::new_inst(ps_cls.clone());

    // Set System.out
    let system = oop::class::load_and_init(b"java/lang/System");
    {
        let cls = system.get_class();
        let fid = cls.get_field_id(&new_br("out"), &new_br("Ljava/io/PrintStream;"), true);
        cls.put_static_field_value(fid, ps_out.clone());
    }

    // Set System.err (same PrintStream)
    {
        let cls = system.get_class();
        let fid = cls.get_field_id(&new_br("err"), &new_br("Ljava/io/PrintStream;"), true);
        cls.put_static_field_value(fid, ps_out.clone());
    }
}
