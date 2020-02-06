use crate::classfile::consts::{
    J_ARRAY_INDEX_OUT_OF_BOUNDS, J_CLASS, J_CLASS_NOT_FOUND, J_CLONEABLE, J_INPUT_STREAM,
    J_INTERNAL_ERROR, J_IOEXCEPTION, J_NPE, J_OBJECT, J_PRINT_STREAM, J_SECURITY_MANAGER,
    J_SERIALIZABLE, J_STRING, J_SYSTEM, J_THREAD, J_THREAD_GROUP,
};
use crate::oop::{self, ClassRef, OopDesc};
use crate::runtime::{self, require_class3, JavaThread};
use crate::util;
use std::sync::Arc;

pub fn initialize_jvm(jt: &mut JavaThread) {
    initialize_vm_structs(jt);

    let thread_cls = do_init(J_THREAD, jt);
    let thread_group_cls = do_init(J_THREAD_GROUP, jt);

    let init_thread_oop = OopDesc::new_inst(thread_cls.clone());
    {
        let mut cls = thread_cls.lock().unwrap();
        //todo: getNativeHandler
        let id = util::new_id_ref2(J_THREAD, b"eetop", b"J");
        cls.put_field_value2(init_thread_oop.clone(), id, oop::OopDesc::new_long(0));
        //todo: define java::lang::ThreadPriority::NORMAL_PRIORITY
        let id = util::new_id_ref2(J_THREAD, b"priority", b"I");
        cls.put_field_value2(init_thread_oop.clone(), id, oop::OopDesc::new_int(5));
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
        let id = util::new_id_ref2(J_THREAD, b"group", b"Ljava/lang/ThreadGroup;");
        cls.put_field_value2(init_thread_oop.clone(), id, main_thread_group.clone());
    }

    let _ = do_init(J_INPUT_STREAM, jt);
    let _ = do_init(J_PRINT_STREAM, jt);
    let _ = do_init(J_SECURITY_MANAGER, jt);

    // Construct the main thread group
    let args = vec![
        main_thread_group.clone(),
        oop::consts::get_null(),
        system_thread_group,
        OopDesc::new_str(Arc::new(Box::new(Vec::from("main")))),
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
        OopDesc::new_str(Arc::new(Box::new(Vec::from("main")))),
    ];
    runtime::java_call::invoke_ctor(
        jt,
        thread_cls.clone(),
        b"(Ljava/lang/ThreadGroup;Ljava/lang/String;)V",
        args,
    );

    //todo: hackJavaClasses

    let init_system_classes_method = {
        let cls = require_class3(None, J_SYSTEM).unwrap();
        let cls = cls.lock().unwrap();
        let id = util::new_id_ref(b"initializeSystemClass", b"()V");
        cls.get_static_method(id).unwrap()
    };
    let mut jc =
        runtime::java_call::JavaCall::new_with_args(jt, init_system_classes_method, vec![]);
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
        let id = util::new_id_ref2(J_CLASS, b"useCaches", b"Z");
        cls.put_static_field_value2(id, OopDesc::new_int(0));
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
