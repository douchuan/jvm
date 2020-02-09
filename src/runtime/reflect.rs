use crate::classfile::{consts as cls_const, consts::J_FIELD};
use crate::runtime;
use crate::util;
use crate::oop::{ClassRef, FieldIdRef, OopDesc, OopRef};

pub fn new_java_field_object(fir: FieldIdRef) -> OopRef {
    let field_cls = runtime::require_class3(None, J_FIELD).unwrap();

    let field_oop = OopDesc::new_inst(field_cls.clone());
    let clazz = {
        fir.field.class.lock().unwrap().get_mirror()
    };

    {
        let cls = field_cls.lock().unwrap();

        vec![
            ("clazz", "Ljava/lang/Class;", clazz),
            ("modifiers", "I", OopDesc::new_int(fir.field.acc_flags as i32)),
            ("slot", "I", OopDesc::new_int(fir.offset as i32)),
            ("name", "Ljava/lang/String;", OopDesc::new_str(fir.field.name.clone()))
        ].iter().for_each(|(name, desc, v)| {
            let id = cls.get_field_id(name.as_bytes(), desc.as_bytes(), false);
            cls.put_field_value(field_oop.clone(), id, v.clone());
        });

        //todo: signature, type
    }

    field_oop
}

