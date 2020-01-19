use crate::classfile::{access_flags::*, attr_info, constant_pool, consts, types::*, FieldInfo};
use crate::oop::{consts as oop_consts, ClassObject, ClassRef, Oop, OopDesc, ValueType};
use crate::runtime::{require_class2, JavaThreadRef};
use crate::util::{self, PATH_DELIMITER};
use std::ops::Deref;
use std::sync::Arc;

pub type FieldIdRef = Arc<FieldId>;

pub fn get_field_ref(
    thread: JavaThreadRef,
    cp: &ConstantPool,
    idx: usize,
    is_static: bool,
) -> FieldIdRef {
    let (class_index, name_and_type_index) = constant_pool::get_field_ref(cp, idx);

    //load Field's Class, then init it
    let class = require_class2(class_index, cp).unwrap();
    let id = {
        let mut class = class.lock().unwrap();
        class.init_class(thread);

        let (name, typ) = constant_pool::get_name_and_type(cp, name_and_type_index as usize);
        let name = name.unwrap();
        let typ = typ.unwrap();

        Arc::new(
            vec![
                class.name.deref().as_slice(),
                typ.deref().as_slice(),
                name.deref().as_slice(),
            ]
            .join(PATH_DELIMITER),
        )
    };

    let class = class.lock().unwrap();
    class.get_field_id(id, is_static)
}

#[derive(Debug, Clone)]
pub struct FieldId {
    pub offset: usize,
    pub field: Field,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub class: ClassRef,
    name: BytesRef,
    desc: BytesRef,
    id: BytesRef,

    acc_flags: U2,

    pub value_type: ValueType,

    pub attr_constant_value: Option<Arc<OopDesc>>,
}

impl Field {
    pub fn new(cp: &ConstantPool, fi: &FieldInfo, class: ClassRef) -> Self {
        let name = constant_pool::get_utf8(cp, fi.name_index as usize).unwrap();
        let desc = constant_pool::get_utf8(cp, fi.desc_index as usize).unwrap();
        let value_type = desc.first().unwrap().into();
        let id = {
            let class = class.lock().unwrap();
            vec![class.name.as_slice(), desc.as_slice(), name.as_slice()].join(PATH_DELIMITER)
        };
        //        info!("id = {}", String::from_utf8_lossy(id.as_slice()));
        let id = Arc::new(Vec::from(id));
        let acc_flags = fi.acc_flags;

        let mut attr_constant_value = None;
        fi.attrs.iter().for_each(|a| {
            if let attr_info::AttrType::ConstantValue {
                constant_value_index,
            } = a
            {
                match cp.get(*constant_value_index as usize) {
                    Some(constant_pool::ConstantType::Long { v }) => {
                        let v =
                            i64::from_be_bytes([v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]]);
                        let v = OopDesc::new_long(v);
                        attr_constant_value = Some(v);
                    }
                    Some(constant_pool::ConstantType::Float { v }) => {
                        let v = u32::from_be_bytes([v[0], v[1], v[2], v[3]]);
                        let v = f32::from_bits(v);
                        let v = OopDesc::new_float(v);
                        attr_constant_value = Some(v);
                    }
                    Some(constant_pool::ConstantType::Double { v }) => {
                        let v =
                            u64::from_be_bytes([v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]]);
                        let v = f64::from_bits(v);
                        let v = OopDesc::new_double(v);
                        attr_constant_value = Some(v);
                    }
                    Some(constant_pool::ConstantType::Integer { v }) => {
                        let v = i32::from_be_bytes([v[0], v[1], v[2], v[3]]);
                        let v = OopDesc::new_int(v);
                        attr_constant_value = Some(v);
                    }
                    Some(constant_pool::ConstantType::String { string_index }) => {
                        if let Some(v) = constant_pool::get_utf8(cp, *string_index as usize) {
                            let v = OopDesc::new_str(v);
                            attr_constant_value = Some(v);
                        }
                    }
                    _ => unreachable!(),
                }
            }
        });

        Self {
            class,
            name,
            desc,
            id,
            acc_flags,
            value_type,
            attr_constant_value,
        }
    }

    pub fn get_id(&self) -> BytesRef {
        self.id.clone()
    }

    pub fn is_public(&self) -> bool {
        (self.acc_flags & ACC_PUBLIC) == ACC_PUBLIC
    }

    pub fn is_private(&self) -> bool {
        (self.acc_flags & ACC_PRIVATE) == ACC_PRIVATE
    }

    pub fn is_protected(&self) -> bool {
        (self.acc_flags & ACC_PROTECTED) == ACC_PROTECTED
    }

    pub fn is_final(&self) -> bool {
        (self.acc_flags & ACC_FINAL) == ACC_FINAL
    }

    pub fn is_static(&self) -> bool {
        (self.acc_flags & ACC_STATIC) == ACC_STATIC
    }

    pub fn is_volatile(&self) -> bool {
        (self.acc_flags & ACC_VOLATILE) == ACC_VOLATILE
    }

    pub fn get_constant_value(&self) -> Arc<OopDesc> {
        match self.value_type {
            ValueType::BYTE
            | ValueType::BOOLEAN
            | ValueType::CHAR
            | ValueType::SHORT
            | ValueType::INT => oop_consts::get_int0(),
            ValueType::LONG => oop_consts::get_long0(),
            ValueType::FLOAT => oop_consts::get_float0(),
            ValueType::DOUBLE => oop_consts::get_double0(),
            ValueType::OBJECT | ValueType::ARRAY => oop_consts::get_null(),
            _ => unreachable!(),
        }
    }

    pub fn get_attr_constant_value(&self) -> Option<Arc<OopDesc>> {
        self.attr_constant_value.clone()
    }
}
