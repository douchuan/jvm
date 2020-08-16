use crate::oop;
use crate::oop::Class;
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
    let thread_group_cls = oop::class::load_and_init(J_THREAD_GROUP);

    let init_thread_oop = oop::Oop::new_inst(thread_cls.clone());
    {
        let mut cls = thread_cls.get_mut_class();
        //todo: getNativeHandler
        //        let id = util::new_field_id(J_THREAD, b"eetop", b"J");
        //        cls.put_field_value2(init_thread_oop.clone(), id, oop::OopDesc::new_long(0));
        //todo: define java::lang::ThreadPriority::NORMAL_PRIORITY
        let id = cls.get_field_id(&new_br("priority"), &new_br("I"), false);
        Class::put_field_value(init_thread_oop.extract_ref(), id, oop::Oop::new_int(5));
    }

    // JavaMainThread is created with java_thread_obj none
    // Now we have created a thread for it.
    let jt = runtime::thread::current_java_thread();
    jt.write()
        .unwrap()
        .set_java_thread_obj(init_thread_oop.clone());

    // Create and construct the system thread group.
    let system_thread_group = oop::Oop::new_inst(thread_group_cls.clone());
    let args = vec![system_thread_group.clone()];
    runtime::invoke::invoke_ctor(thread_group_cls.clone(), new_br("()V"), args);

    let main_thread_group = oop::Oop::new_inst(thread_group_cls.clone());

    {
        let mut cls = thread_cls.get_mut_class();
        let id = cls.get_field_id(&new_br("group"), &new_br("Ljava/lang/ThreadGroup;"), false);
        Class::put_field_value(init_thread_oop.extract_ref(), id, main_thread_group.clone());
    }

    let _ = oop::class::load_and_init(J_INPUT_STREAM);
    let _ = oop::class::load_and_init(J_PRINT_STREAM);
    let _ = oop::class::load_and_init(J_SECURITY_MANAGER);

    // Construct the main thread group
    let args = vec![
        main_thread_group.clone(),
        oop::consts::get_null(),
        system_thread_group,
        util::oop::new_java_lang_string2("main"),
    ];
    runtime::invoke::invoke_ctor(
        thread_group_cls,
        new_br("(Ljava/lang/Void;Ljava/lang/ThreadGroup;Ljava/lang/String;)V"),
        args,
    );

    //todo: disable sun.security.util.Debug for the following operations
    //need to impl java_security_accesscontroller
    //    let sun_debug_cls = do_init(b"sun/security/util/Debug", jt);

    let args = vec![
        init_thread_oop,
        main_thread_group,
        util::oop::new_java_lang_string2("main"),
    ];
    runtime::invoke::invoke_ctor(
        thread_cls,
        new_br("(Ljava/lang/ThreadGroup;Ljava/lang/String;)V"),
        args,
    );

    hack_classes();

    let init_system_classes_method = {
        let cls = require_class3(None, J_SYSTEM).unwrap();
        let cls = cls.get_class();
        cls.get_static_method(&new_br("initializeSystemClass"), &new_br("()V"))
            .unwrap()
    };
    let mut jc = runtime::invoke::JavaCall::new_with_args(init_system_classes_method, vec![]);
    jc.invoke(None, false);

    //todo: re-enable sun.security.util.Debug

    //setup security
    let _ = oop::class::load_and_init(b"sun/security/provider/Sun");
    let _ = oop::class::load_and_init(b"sun/security/rsa/SunRsaSign");
    let _ = oop::class::load_and_init(b"com/sun/net/ssl/internal/ssl/Provider");
}

fn initialize_vm_structs() {
    let class_obj = oop::class::load_and_init(J_CLASS);
    native::java_lang_Class::create_delayed_mirrors();
    native::java_lang_Class::create_delayed_ary_mirrors();

    let _ = oop::class::load_and_init(J_OBJECT);
    let string_cls = oop::class::load_and_init(J_STRING);
    {
        let cls = string_cls.get_class();
        let fir = cls.get_field_id(&new_br("value"), &new_br("[C"), false);
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

    //todo:
    //java::lang::reflect::Constructor::initialize
    //java::lang::reflect::Method::initialize

    {
        let mut cls = class_obj.get_mut_class();
        let id = cls.get_field_id(&new_br("useCaches"), &new_br("Z"), true);
        cls.put_static_field_value(id, oop::Oop::new_int(1));
    }
}

fn hack_classes() {
    let charset_cls = oop::class::load_and_init(b"java/nio/charset/Charset");
    let ascii_charset_cls = oop::class::load_and_init(b"sun/nio/cs/US_ASCII");

    let ascii_inst = oop::Oop::new_inst(ascii_charset_cls.clone());
    let args = vec![ascii_inst.clone()];
    runtime::invoke::invoke_ctor(ascii_charset_cls, new_br("()V"), args);

    {
        let mut cls = charset_cls.get_mut_class();
        let id = cls.get_field_id(
            &new_br("defaultCharset"),
            &new_br("Ljava/nio/charset/Charset;"),
            true,
        );
        cls.put_static_field_value(id, ascii_inst);
    }

    let encoder = oop::class::load_and_init(b"sun/nio/cs/StreamEncoder");
    {
        let mut cls = encoder.get_mut_class();
        cls.hack_as_native(b"forOutputStreamWriter", b"(Ljava/io/OutputStream;Ljava/lang/Object;Ljava/lang/String;)Lsun/nio/cs/StreamEncoder;");
    }

    let system = oop::class::load_and_init(b"java/lang/System");

    {
        let mut cls = system.get_mut_class();
        cls.hack_as_native(b"load", b"(Ljava/lang/String;)V");

        //todo: support load lib
        cls.hack_as_native(b"loadLibrary", b"(Ljava/lang/String;)V");

        //fixme: rm, just for debug
        //        let id = util::new_method_id(b"getProperty", b"(Ljava/lang/String;)Ljava/lang/String;");
        //        cls.hack_as_native(id);
    }

    /*
    let mut mir = {
        let cls = encoder.lock().unwrap();
        let id = util::new_method_id(b"forOutputStreamWriter", b"(Ljava/io/OutputStream;Ljava/lang/Object;Ljava/lang/String;)Lsun/nio/cs/StreamEncoder;");
        cls.get_static_method(id).unwrap()
    };
    */
}
