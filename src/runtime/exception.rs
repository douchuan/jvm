use crate::oop::{self, Oop};
use crate::runtime::{self, require_class3, JavaThread};
use crate::types::BytesRef;
use crate::util;

pub fn new(jt: &mut JavaThread, name: &[u8], msg: Option<String>) -> Oop {
    let cls = match require_class3(None, name) {
        Some(cls) => cls,
        None => panic!("ClassNotFound: {}", String::from_utf8_lossy(name)),
    };

    {
        let mut cls = cls.write().unwrap();
        cls.init_class(jt);
        //                trace!("finish init_class: {}", String::from_utf8_lossy(*c));
    }
    oop::class::init_class_fully(jt, cls.clone());

    let ex = Oop::new_inst(cls.clone());

    //invoke ctor
    match &msg {
        Some(msg) => {
            //with 'String' arg ctor
            let msg = util::oop::new_java_lang_string2(jt, msg);
            let args = vec![ex.clone(), msg];
            runtime::java_call::invoke_ctor(jt, cls.clone(), b"(Ljava/lang/String;)V", args);
        }
        None => {
            //No arg ctor
            let args = vec![ex.clone()];
            runtime::java_call::invoke_ctor(jt, cls.clone(), b"()V", args);
        }
    }

    ex
}
