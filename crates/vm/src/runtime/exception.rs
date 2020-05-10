use crate::oop::{self, Oop};
use crate::runtime::{self, require_class3};
use crate::types::JavaThreadRef;
use crate::util;

pub fn new(name: &[u8], msg: Option<String>) -> Oop {
    let cls = match require_class3(None, name) {
        Some(cls) => cls,
        None => panic!("ClassNotFound: {}", String::from_utf8_lossy(name)),
    };

    {
        let mut cls = cls.write().unwrap();
        cls.init_class();
        //                trace!("finish init_class: {}", String::from_utf8_lossy(*c));
    }
    oop::class::init_class_fully(cls.clone());

    let ex = Oop::new_inst(cls.clone());

    //invoke ctor
    match &msg {
        Some(msg) => {
            //with 'String' arg ctor
            let msg = util::oop::new_java_lang_string2(msg);
            let args = vec![ex.clone(), msg];
            runtime::invoke::invoke_ctor(cls.clone(), b"(Ljava/lang/String;)V", args);
        }
        None => {
            //No arg ctor
            let args = vec![ex.clone()];
            runtime::invoke::invoke_ctor(cls.clone(), b"()V", args);
        }
    }

    ex
}

pub fn meet_ex(cls_name: &'static [u8], msg: Option<String>) {
    let jt = runtime::thread::current_java_thread();
    {
        let jt = jt.read().unwrap();
        let frame = jt.frames.last().unwrap();
        let frame = frame.try_read().unwrap();
        frame.area.write().unwrap().ex_here = true;
    }

    let ex = new(cls_name, msg);
    jt.write().unwrap().set_ex(ex);
}
