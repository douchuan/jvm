use crate::oop::{self, ClassRef};
use crate::runtime::{self, JavaThread};
use crate::classfile::consts::{J_CLASS, J_OBJECT, J_STRING, J_CLONEABLE, J_SERIALIZABLE, J_NPE, J_ARRAY_INDEX_OUT_OF_BOUNDS, J_CLASS_NOT_FOUND, J_INTERNAL_ERROR, J_IOEXCEPTION, J_THREAD, J_THREAD_GROUP};

pub fn initialize_jvm(jt: &mut JavaThread) {

    initialize_vm_structs(jt);

    let thread_cls = do_init(J_THREAD, jt);
    let thread_group_cls = do_init(J_THREAD_GROUP, jt);

    let init_thread_oop = oop::OopDesc::new_inst(thread_cls.clone());
    {
        let mut cls = thread_cls.lock().unwrap();
        //todo: getNativeHandler
        cls.put_field_value2(init_thread_oop.clone(), J_THREAD, b"J", b"eetop", oop::OopDesc::new_long(0));
        //todo: define java::lang::ThreadPriority::NORMAL_PRIORITY
        cls.put_field_value2(init_thread_oop.clone(), J_THREAD, b"I", b"priority", oop::OopDesc::new_int(5));
    }

    // JavaMainThread is created with java_thread_obj none
    // Now we have created a thread for it.
    jt.set_java_thread_obj(init_thread_oop);

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
        cls.put_static_field_value2(J_CLASS, b"Z", b"useCaches", oop::OopDesc::new_int(0));
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