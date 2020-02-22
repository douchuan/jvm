use crate::classfile::consts::{
    J_ARRAY_INDEX_OUT_OF_BOUNDS, J_CLASS, J_CLASS_NOT_FOUND, J_CLONEABLE, J_FIELD, J_INPUT_STREAM,
    J_INTERNAL_ERROR, J_IOEXCEPTION, J_METHOD_CTOR, J_NPE, J_OBJECT, J_PRINT_STREAM,
    J_SECURITY_MANAGER, J_SERIALIZABLE, J_STRING, J_SYSTEM, J_THREAD, J_THREAD_GROUP, J_THROWABLE,
};
use crate::native;
use crate::oop::{self, OopDesc};
use crate::runtime::{self, require_class3, JavaThread};
use crate::util;
use std::borrow::BorrowMut;
use std::sync::Arc;

pub fn initialize_jvm(jt: &mut JavaThread) {
    initialize_vm_structs(jt);

    let thread_cls = oop::class::load_and_init(jt, J_THREAD);
    let thread_group_cls = oop::class::load_and_init(jt, J_THREAD_GROUP);

    let init_thread_oop = OopDesc::new_inst(thread_cls.clone());
    {
        let mut cls = thread_cls.lock().unwrap();
        //todo: getNativeHandler
        //        let id = util::new_field_id(J_THREAD, b"eetop", b"J");
        //        cls.put_field_value2(init_thread_oop.clone(), id, oop::OopDesc::new_long(0));
        //todo: define java::lang::ThreadPriority::NORMAL_PRIORITY
        let id = cls.get_field_id(b"priority", b"I", false);
        cls.put_field_value(init_thread_oop.clone(), id, oop::OopDesc::new_int(5));
    }

    // JavaMainThread is created with java_thread_obj none
    // Now we have created a thread for it.
    jt.set_java_thread_obj(init_thread_oop.clone());

    // Create and construct the system thread group.
    let system_thread_group = OopDesc::new_inst(thread_group_cls.clone());
    let args = vec![system_thread_group.clone()];
    runtime::java_call::invoke_ctor(jt, thread_group_cls.clone(), b"()V", args);

    let main_thread_group = OopDesc::new_inst(thread_group_cls.clone());

    {
        let mut cls = thread_cls.lock().unwrap();
        let id = cls.get_field_id(b"group", b"Ljava/lang/ThreadGroup;", false);
        cls.put_field_value(init_thread_oop.clone(), id, main_thread_group.clone());
    }

    let _ = oop::class::load_and_init(jt, J_INPUT_STREAM);
    let _ = oop::class::load_and_init(jt, J_PRINT_STREAM);
    let _ = oop::class::load_and_init(jt, J_SECURITY_MANAGER);

    // Construct the main thread group
    let args = vec![
        main_thread_group.clone(),
        oop::consts::get_null(),
        system_thread_group,
        util::oop::new_java_lang_string2(jt, "main"),
    ];
    runtime::java_call::invoke_ctor(
        jt,
        thread_group_cls.clone(),
        b"(Ljava/lang/Void;Ljava/lang/ThreadGroup;Ljava/lang/String;)V",
        args,
    );

    //todo: disable sun.security.util.Debug for the following operations
    //need to impl java_security_accesscontroller
    //    let sun_debug_cls = do_init(b"sun/security/util/Debug", jt);

    let args = vec![
        init_thread_oop,
        main_thread_group,
        util::oop::new_java_lang_string2(jt, "main"),
    ];
    runtime::java_call::invoke_ctor(
        jt,
        thread_cls.clone(),
        b"(Ljava/lang/ThreadGroup;Ljava/lang/String;)V",
        args,
    );

    hack_classes(jt);

    let init_system_classes_method = {
        let cls = require_class3(None, J_SYSTEM).unwrap();
        let cls = cls.lock().unwrap();
        let id = util::new_method_id(b"initializeSystemClass", b"()V");
        cls.get_static_method(id).unwrap()
    };
    let mut jc =
        runtime::java_call::JavaCall::new_with_args(jt, init_system_classes_method, vec![]);
    let mut stack = runtime::stack::Stack::new(0);
    jc.invoke(jt, &mut stack, false);

    //todo: re-enable sun.security.util.Debug

    //setup security
    let _ = oop::class::load_and_init(jt, b"sun.security.provider.Sun");
}

fn initialize_vm_structs(jt: &mut JavaThread) {
    let class_obj = oop::class::load_and_init(jt, J_CLASS);
    native::java_lang_Class::create_delayed_mirrors();
    native::java_lang_Class::create_delayed_ary_mirrors();

    let _ = oop::class::load_and_init(jt, J_OBJECT);
    let string_cls = oop::class::load_and_init(jt, J_STRING);
    {
        let cls = string_cls.lock().unwrap();
        let fir = cls.get_field_id(b"value", b"[C", false);
        util::oop::set_java_lang_string_value_offset(fir.offset);
    }

    let _ = oop::class::load_and_init(jt, J_CLONEABLE);
    let _ = oop::class::load_and_init(jt, J_SERIALIZABLE);
    let _ = oop::class::load_and_init(jt, J_NPE);
    let _ = oop::class::load_and_init(jt, J_ARRAY_INDEX_OUT_OF_BOUNDS);
    let _ = oop::class::load_and_init(jt, J_CLASS_NOT_FOUND);
    let _ = oop::class::load_and_init(jt, J_INTERNAL_ERROR);
    let _ = oop::class::load_and_init(jt, J_IOEXCEPTION);
    let _ = oop::class::load_and_init(jt, J_FIELD);
    let _ = oop::class::load_and_init(jt, J_METHOD_CTOR);
    let _ = oop::class::load_and_init(jt, J_THROWABLE);

    //todo:
    //java::lang::reflect::Constructor::initialize
    //java::lang::reflect::Method::initialize

    {
        let mut cls = class_obj.lock().unwrap();
        let id = cls.get_field_id(b"useCaches", b"Z", true);
        cls.put_static_field_value(id, OopDesc::new_int(1));
    }
}

fn hack_classes(jt: &mut JavaThread) {
    let charset_cls = oop::class::load_and_init(jt, b"java/nio/charset/Charset");
    let ascii_charset_cls = oop::class::load_and_init(jt, b"sun/nio/cs/US_ASCII");

    let ascii_inst = OopDesc::new_inst(ascii_charset_cls.clone());
    let args = vec![ascii_inst.clone()];
    runtime::java_call::invoke_ctor(jt, ascii_charset_cls.clone(), b"()V", args);

    {
        let mut cls = charset_cls.lock().unwrap();
        let id = cls.get_field_id(b"defaultCharset", b"Ljava/nio/charset/Charset;", true);
        cls.put_static_field_value(id, ascii_inst);
    }

    let encoder = oop::class::load_and_init(jt, b"sun/nio/cs/StreamEncoder");
    {
        let mut cls = encoder.lock().unwrap();
        let id = util::new_method_id(b"forOutputStreamWriter", b"(Ljava/io/OutputStream;Ljava/lang/Object;Ljava/lang/String;)Lsun/nio/cs/StreamEncoder;");
        cls.hack_as_native(id);
    }

    let system = oop::class::load_and_init(jt, b"java/lang/System");

    {
        let mut cls = system.lock().unwrap();
        let id = util::new_method_id(b"load", b"(Ljava/lang/String;)V");
        cls.hack_as_native(id);

        //todo: support load lib
        let id = util::new_method_id(b"loadLibrary", b"(Ljava/lang/String;)V");
        cls.hack_as_native(id);

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
