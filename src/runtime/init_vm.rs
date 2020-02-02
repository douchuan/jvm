use crate::oop::{self, ClassRef, OopDesc};
use crate::runtime::{self, JavaThread, require_class3};
use crate::classfile::consts::{J_CLASS, J_OBJECT, J_STRING, J_CLONEABLE, J_SERIALIZABLE, J_NPE, J_ARRAY_INDEX_OUT_OF_BOUNDS, J_CLASS_NOT_FOUND, J_INTERNAL_ERROR, J_IOEXCEPTION, J_THREAD, J_THREAD_GROUP, J_INPUT_STREAM, J_PRINT_STREAM, J_SECURITY_MANAGER, J_SYSTEM};
use std::sync::Arc;

pub fn initialize_jvm(jt: &mut JavaThread) {

    initialize_vm_structs(jt);

    let thread_cls = do_init(J_THREAD, jt);
    let thread_group_cls = do_init(J_THREAD_GROUP, jt);

    let init_thread_oop = OopDesc::new_inst(thread_cls.clone());
    {
        let mut cls = thread_cls.lock().unwrap();
        //todo: getNativeHandler
        cls.put_field_value2(init_thread_oop.clone(), J_THREAD, b"J", b"eetop", oop::OopDesc::new_long(0));
        //todo: define java::lang::ThreadPriority::NORMAL_PRIORITY
        cls.put_field_value2(init_thread_oop.clone(), J_THREAD, b"I", b"priority", oop::OopDesc::new_int(5));
    }

    // JavaMainThread is created with java_thread_obj none
    // Now we have created a thread for it.
    jt.set_java_thread_obj(init_thread_oop.clone());

    // Create and construct the system thread group.
    let system_thread_group = OopDesc::new_inst(thread_group_cls.clone());
    let ctor = {
        let cls = thread_group_cls.lock().unwrap();
        cls.get_this_class_method(b"()V", b"<init>").unwrap()
    };
    let mut jc = runtime::java_call::JavaCall::new_with_args(jt, ctor, vec![system_thread_group.clone()]);
    let mut stack = runtime::stack::Stack::new(0);
    jc.invoke(jt, &mut stack);

    let main_thread_group = OopDesc::new_inst(thread_group_cls.clone());

    {
        let mut cls = thread_cls.lock().unwrap();
        cls.put_field_value2(init_thread_oop.clone(), J_THREAD, b"Ljava/lang/ThreadGroup;", b"group", main_thread_group.clone());
    }

    let _ = do_init(J_INPUT_STREAM, jt);
    let _ = do_init(J_PRINT_STREAM, jt);
    let _ = do_init(J_SECURITY_MANAGER, jt);

    // Construct the main thread group
    // use get_this_class_method() to get a private method
    warn!("xxxxx thread_group ctor");
    let ctor = {
        let cls = thread_group_cls.lock().unwrap();
        cls.get_this_class_method(b"(Ljava/lang/Void;Ljava/lang/ThreadGroup;Ljava/lang/String;)V", b"<init>").unwrap()
    };
    let mut args = vec![
        main_thread_group.clone(),
        oop::consts::get_null(),
        system_thread_group,
        OopDesc::new_str(Arc::new(Vec::from("main")))
    ];
    let mut jc = runtime::java_call::JavaCall::new_with_args(jt, ctor, args);
    let mut stack = runtime::stack::Stack::new(0);
    jc.invoke(jt, &mut stack);

    //todo: disable sun.security.util.Debug for the following operations
    //need to impl java_security_accesscontroller
//    let sun_debug_cls = do_init(b"sun/security/util/Debug", jt);

    warn!("xxxxx thread ctor");
    let ctor = {
        let cls = thread_cls.lock().unwrap();
        cls.get_this_class_method(b"(Ljava/lang/ThreadGroup;Ljava/lang/String;)V", b"<init>").unwrap()
    };
    let args = vec![
        init_thread_oop,
        main_thread_group,
        OopDesc::new_str(Arc::new(Vec::from("main")))
    ];
    let mut jc = runtime::java_call::JavaCall::new_with_args(jt, ctor, args);
    let mut stack = runtime::stack::Stack::new(0);
    jc.invoke(jt, &mut stack);
    trace!("xxxxx 2");

    //todo: hackJavaClasses

    let init_system_classes_method = {
        let cls = require_class3(None, J_SYSTEM).unwrap();
        let cls = cls.lock().unwrap();
        cls.get_static_method(b"()V", b"initializeSystemClass").unwrap()
    };
    let mut jc = runtime::java_call::JavaCall::new_with_args(jt, init_system_classes_method, vec![]);
    let mut stack = runtime::stack::Stack::new(0);
    jc.invoke(jt, &mut stack);

    //todo: re-enable sun.security.util.Debug
}

fn initialize_vm_structs(jt: &mut JavaThread) {
    //todo: java::lang::Class::initialize
    let class_obj = do_init(J_CLASS, jt);
    //todo:
//        java::lang::Class::mirrorCoreAndDelayedClasses();
//        java::lang::Class::mirrorDelayedArrayClasses();
    let _ = do_init(J_OBJECT, jt);
    let _ = do_init(J_STRING, jt);
    let _ = do_init(J_CLONEABLE, jt);
    let _ = do_init(J_SERIALIZABLE, jt);
    let _ = do_init(J_NPE, jt);
    let _ = do_init(J_ARRAY_INDEX_OUT_OF_BOUNDS, jt);
    let _ = do_init(J_CLASS_NOT_FOUND, jt);
    let _ = do_init(J_INTERNAL_ERROR, jt);
    let _ = do_init(J_IOEXCEPTION, jt);

    //todo:
    //java::lang::reflect::Constructor::initialize
    //java::lang::reflect::Method::initialize

    {
        let mut cls = class_obj.lock().unwrap();
        cls.put_static_field_value2(J_CLASS, b"Z", b"useCaches", OopDesc::new_int(0));
    }
}

fn do_init(name: &[u8], jt: &mut JavaThread) -> ClassRef {
    let class = runtime::require_class3(None, name);
    let class = class.unwrap();
    {
        let mut class = class.lock().unwrap();
        class.init_class(jt);
        //                trace!("finish init_class: {}", String::from_utf8_lossy(*c));
    }
    oop::class::init_class_fully(jt, class.clone());
    //            trace!("finish init_class_fully: {}", String::from_utf8_lossy(*c));

    class
}