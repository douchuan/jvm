use crate::classfile::{consts as cls_const, consts::J_FIELD};
use crate::native::java_lang_Class;
use crate::runtime::{self, require_class3};
use crate::util;
use crate::oop::{ClassRef, FieldIdRef, OopDesc, OopRef, ValueType};

pub fn new_java_field_object(fir: FieldIdRef) -> OopRef {
    let field_cls = runtime::require_class3(None, J_FIELD).unwrap();

    let field_oop = OopDesc::new_inst(field_cls.clone());
    let clazz = {
        fir.field.class.lock().unwrap().get_mirror()
    };

    let typ_mirror = match fir.field.value_type {
        ValueType::VOID => unreachable!(),
        ValueType::OBJECT => {
            let len = fir.field.desc.len();
            let name = &fir.field.desc.as_slice()[1..len-1];
            let cls = require_class3(None, name).unwrap();
            let cls = cls.lock().unwrap();
            cls.get_mirror()
        },
        ValueType::ARRAY => {
            let cls = require_class3(None, fir.field.desc.as_slice()).unwrap();
            let cls = cls.lock().unwrap();
            cls.get_mirror()
        },
        t => {
            let key = t.into();
            let key = String::from_utf8_lossy(key).to_string();
            java_lang_Class::get_primitive_class_mirror(key.as_str()).unwrap()
        }
    };

    let signature = OopDesc::new_str(fir.field.desc.clone());

    {
        let cls = field_cls.lock().unwrap();

        vec![
            ("clazz", "Ljava/lang/Class;", clazz),
            ("modifiers", "I", OopDesc::new_int(fir.field.acc_flags as i32)),
            ("slot", "I", OopDesc::new_int(fir.offset as i32)),
            ("name", "Ljava/lang/String;", OopDesc::new_str(fir.field.name.clone())),
            ("type", "Ljava/lang/Class;", typ_mirror),
            ("signature", "Ljava/lang/String;", signature),
        ].iter().for_each(|(name, desc, v)| {
            let id = cls.get_field_id(name.as_bytes(), desc.as_bytes(), false);
            cls.put_field_value(field_oop.clone(), id, v.clone());
        });
    }

    field_oop
}

