use crate::classfile::{consts as cls_const, consts::J_FIELD};
use crate::native::java_lang_Class;
use crate::oop::{self, ClassRef, FieldIdRef, OopDesc, OopRef, ValueType};
use crate::runtime::{self, require_class3, JavaThread};
use crate::util;

pub fn new_java_field_object(jt: &mut JavaThread, fir: FieldIdRef) -> OopRef {
    let field_cls = runtime::require_class3(None, J_FIELD).unwrap();

    let field_oop = OopDesc::new_inst(field_cls.clone());

    let clazz = { fir.field.class.lock().unwrap().get_mirror() };

    let typ_mirror = match fir.field.value_type {
        ValueType::VOID => unreachable!(),
        ValueType::OBJECT => {
            let len = fir.field.desc.len();
            let name = &fir.field.desc.as_slice()[1..len - 1];
            let cls = require_class3(None, name).unwrap();
            let cls = cls.lock().unwrap();
            cls.get_mirror()
        }
        ValueType::ARRAY => {
            let cls = require_class3(None, fir.field.desc.as_slice()).unwrap();
            let cls = cls.lock().unwrap();
            cls.get_mirror()
        }
        t => {
            let key = t.into();
            let key = String::from_utf8_lossy(key).to_string();
            java_lang_Class::get_primitive_class_mirror(key.as_str()).unwrap()
        }
    };

    let signature = OopDesc::new_str(fir.field.desc.clone());

    let mut desc = Vec::new();
    desc.push(b'(');
    let mut args: Vec<OopRef> = vec![
        ("clazz", "Ljava/lang/Class;", clazz),
        (
            "name",
            "Ljava/lang/String;",
            OopDesc::new_str(fir.field.name.clone()),
        ),
        ("type", "Ljava/lang/Class;", typ_mirror),
        (
            "modifiers",
            "I",
            OopDesc::new_int(fir.field.acc_flags as i32),
        ),
        ("slot", "I", OopDesc::new_int(fir.offset as i32)),
        ("signature", "Ljava/lang/String;", signature),
        ("annotations", "[B", oop::consts::get_null()),
    ]
    .iter()
    .map(|(_, t, v)| {
        desc.extend_from_slice(t.as_bytes());
        v.clone()
    })
    .collect();
    desc.extend_from_slice(")V".as_bytes());

    args.insert(0, field_oop.clone());
    runtime::java_call::invoke_ctor(jt, field_cls, desc.as_slice(), args);

    field_oop
}
