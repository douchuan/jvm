use crate::classfile::{attributes, constant_pool, consts, flags::*, FieldInfo};
use crate::oop::{self, consts as oop_consts, ClassRef, Oop, ValueType};
use crate::runtime::{require_class2, JavaThread};
use crate::types::*;
use crate::util;
use crate::util::PATH_SEP;
use std::ops::Deref;
use std::sync::Arc;

pub fn get_field_ref(
    thread: &mut JavaThread,
    cp: &ConstantPool,
    idx: usize,
    is_static: bool,
) -> FieldIdRef {
    let (class_index, name_and_type_index) = constant_pool::get_field_ref(cp, idx);

    //load Field's Class, then init it
    let class = require_class2(class_index, cp).unwrap_or_else(|| {
        panic!(
            "Unknown field class {:?}",
            cp.get(class_index as usize)
                .expect("Missing item")
                .as_cp_item(cp)
        )
    });
    let (name, desc) = {
        let mut class = class.write().unwrap();
        class.init_class(thread);

        let (name, desc) = constant_pool::get_name_and_type(cp, name_and_type_index as usize);
        let name = name.unwrap();
        let desc = desc.unwrap();

        (name, desc)
    };

    //    trace!("get_field_ref id={}", String::from_utf8_lossy(id.as_slice()));

    oop::class::init_class_fully(thread, class.clone());

    let class = class.read().unwrap();
    class.get_field_id(name.as_slice(), desc.as_slice(), is_static)
}

pub fn build_inited_field_values(class: ClassRef) -> Vec<Oop> {
    let n = {
        let class = class.read().unwrap();
        match &class.kind {
            oop::class::ClassKind::Instance(class_obj) => class_obj.n_inst_fields,
            _ => unreachable!(),
        }
    };

    let mut field_values = Vec::with_capacity(n);
    field_values.resize(n, oop::consts::get_null());

    let mut cur_cls = class.clone();
    loop {
        let cls = cur_cls.clone();
        let cls = cls.read().unwrap();
        match &cls.kind {
            oop::class::ClassKind::Instance(cls_obj) => {
                cls_obj.inst_fields.iter().for_each(|(_, fir)| {
                    match fir.field.value_type {
                        ValueType::BYTE
                        | ValueType::BOOLEAN
                        | ValueType::CHAR
                        | ValueType::SHORT
                        | ValueType::INT => {
                            field_values[fir.offset] = Oop::new_int(0);
                        }
                        ValueType::LONG => {
                            field_values[fir.offset] = Oop::new_long(0);
                        }
                        ValueType::FLOAT => {
                            field_values[fir.offset] = Oop::new_float(0.0);
                        }
                        ValueType::DOUBLE => {
                            field_values[fir.offset] = Oop::new_double(0.0);
                        }
                        ValueType::OBJECT | ValueType::ARRAY => {
                            //ignore, has been inited by NULL
                        }
                        ValueType::VOID => (),
                    }
                });
            }
            _ => unreachable!(),
        }

        if cls.super_class.is_none() {
            break;
        } else {
            cur_cls = cls.super_class.clone().unwrap();
        }
    }

    field_values
}

#[derive(Debug, Clone)]
pub struct FieldId {
    pub offset: usize,
    pub field: Field,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub class: ClassRef,
    pub name: BytesRef,
    pub desc: BytesRef,
    id: BytesRef,

    pub acc_flags: U2,

    pub value_type: ValueType,

    pub attr_constant_value: Option<Oop>,
}

impl Field {
    pub fn new(cp: &ConstantPool, fi: &FieldInfo, class_name: &[u8], class: ClassRef) -> Self {
        let name = constant_pool::get_utf8(cp, fi.name_index as usize).unwrap();
        let desc = constant_pool::get_utf8(cp, fi.desc_index as usize).unwrap();
        let value_type = desc.first().unwrap().into();

        let id = vec![class_name, name.as_slice(), desc.as_slice()].join(PATH_SEP.as_bytes());
        //        info!("id = {}", String::from_utf8_lossy(id.as_slice()));
        let id = Arc::new(id);
        let acc_flags = fi.acc_flags;

        let mut attr_constant_value = None;
        fi.attrs.iter().for_each(|a| {
            if let attributes::Type::ConstantValue {
                constant_value_index,
            } = a
            {
                match cp.get(*constant_value_index as usize) {
                    Some(constant_pool::Type::Long { v }) => {
                        let v =
                            i64::from_be_bytes([v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]]);
                        let v = Oop::new_long(v);
                        attr_constant_value = Some(v);
                    }
                    Some(constant_pool::Type::Float { v }) => {
                        let v = u32::from_be_bytes([v[0], v[1], v[2], v[3]]);
                        let v = f32::from_bits(v);
                        let v = Oop::new_float(v);
                        attr_constant_value = Some(v);
                    }
                    Some(constant_pool::Type::Double { v }) => {
                        let v =
                            u64::from_be_bytes([v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]]);
                        let v = f64::from_bits(v);
                        let v = Oop::new_double(v);
                        attr_constant_value = Some(v);
                    }
                    Some(constant_pool::Type::Integer { v }) => {
                        let v = i32::from_be_bytes([v[0], v[1], v[2], v[3]]);
                        let v = Oop::new_int(v);
                        attr_constant_value = Some(v);
                    }
                    //                    此处没有javathread，如何创建String?
                    Some(constant_pool::Type::String { string_index }) => {
                        if let Some(v) = constant_pool::get_utf8(cp, *string_index as usize) {
                            //                            println!("field const value = {}", String::from_utf8_lossy(v.as_slice()));
                            let v = Oop::new_const_utf8(v);
                            attr_constant_value = Some(v);
                        }
                    }
                    _ => unreachable!(),
                }
            }
        });

        /*
        trace!("field cls={}, name={} desc={}, value_type={:?}, constant_value={:?}",
               String::from_utf8_lossy(class_name),
               String::from_utf8_lossy(name.as_slice()),
               String::from_utf8_lossy(desc.as_slice()),
               value_type,
        attr_constant_value);
        */

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

    pub fn get_constant_value(&self) -> Oop {
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

    pub fn get_attr_constant_value(&self) -> Option<Oop> {
        self.attr_constant_value.clone()
    }
}
