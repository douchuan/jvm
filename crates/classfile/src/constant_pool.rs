use crate::consts::{
    CONSTANT_INTERFACE_METHOD_REF_TAG, CONSTANT_METHOD_REF_TAG, METHOD_NAME_CLINIT,
    METHOD_NAME_INIT,
};
use crate::types::{BytesRef, ConstantPool};
use crate::{MethodSignature, SignatureType};
use fmt::Debug;
use std::fmt;
use std::sync::Arc;

pub fn get_class_name(cp: &ConstantPool, idx: usize) -> Option<BytesRef> {
    match cp.get(idx) {
        Some(Type::Class { name_index }) => get_utf8(cp, *name_index as usize),
        _ => None,
    }
}

pub fn get_field_ref(cp: &ConstantPool, idx: usize) -> (u16, u16) {
    match cp.get(idx) {
        Some(Type::FieldRef {
            class_index,
            name_and_type_index,
        }) => (*class_index, *name_and_type_index),
        _ => unreachable!(),
    }
}

pub fn get_method_ref(cp: &ConstantPool, idx: usize) -> (u8, u16, u16) {
    match cp.get(idx) {
        Some(Type::MethodRef {
            class_index,
            name_and_type_index,
        }) => (CONSTANT_METHOD_REF_TAG, *class_index, *name_and_type_index),
        Some(Type::InterfaceMethodRef {
            class_index,
            name_and_type_index,
        }) => (
            CONSTANT_INTERFACE_METHOD_REF_TAG,
            *class_index,
            *name_and_type_index,
        ),
        _ => unreachable!(),
    }
}

pub fn get_name_and_type(cp: &ConstantPool, idx: usize) -> (Option<BytesRef>, Option<BytesRef>) {
    match cp.get(idx) {
        Some(Type::NameAndType {
            name_index,
            desc_index,
        }) => (
            get_utf8(cp, *name_index as usize),
            get_utf8(cp, *desc_index as usize),
        ),
        _ => (None, None),
    }
}

pub fn get_utf8(cp: &ConstantPool, idx: usize) -> Option<BytesRef> {
    match cp.get(idx) {
        Some(Type::Utf8 { bytes }) => Some(bytes.clone()),
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub enum Type {
    Nop,
    Class {
        name_index: u16,
    },
    FieldRef {
        class_index: u16,
        name_and_type_index: u16,
    },
    MethodRef {
        class_index: u16,
        name_and_type_index: u16,
    },
    InterfaceMethodRef {
        class_index: u16,
        name_and_type_index: u16,
    },
    String {
        string_index: u16,
    },
    Integer {
        v: [u8; 4],
    },
    Float {
        v: [u8; 4],
    },
    Long {
        v: [u8; 8],
    },
    Double {
        v: [u8; 8],
    },
    NameAndType {
        name_index: u16,
        desc_index: u16,
    },
    Utf8 {
        bytes: BytesRef,
    },
    MethodHandle {
        ref_kind: u8,
        ref_index: u16,
    },
    MethodType {
        desc_index: u16,
    },
    InvokeDynamic {
        bootstrap_method_attr_index: u16,
        name_and_type_index: u16,
    },
    Unknown,
}

#[derive(Debug, Clone, Copy)]
pub enum Tag {
    Class,
    FieldRef,
    MethodRef,
    InterfaceMethodRef,
    String,
    Integer,
    Float,
    Long,
    Double,
    NameAndType,
    Utf8,
    MethodHandle,
    MethodType,
    InvokeDynamic,
}

impl From<u8> for Tag {
    fn from(tag: u8) -> Self {
        match tag {
            7 => Tag::Class,
            9 => Tag::FieldRef,
            10 => Tag::MethodRef,
            11 => Tag::InterfaceMethodRef,
            8 => Tag::String,
            3 => Tag::Integer,
            4 => Tag::Float,
            5 => Tag::Long,
            6 => Tag::Double,
            12 => Tag::NameAndType,
            1 => Tag::Utf8,
            15 => Tag::MethodHandle,
            16 => Tag::MethodType,
            18 => Tag::InvokeDynamic,
            _ => unreachable!(),
        }
    }
}

impl Type {
    pub fn as_cp_item<'a, 'b>(&'a self, cp: &'b ConstantPool) -> ConstantPoolItem<'a, 'b> {
        ConstantPoolItem { cp, item: self }
    }
}

pub struct ConstantPoolItem<'item, 'cp> {
    cp: &'cp ConstantPool,
    item: &'item Type,
}

impl Debug for ConstantPoolItem<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.item {
            Type::Nop => f.debug_struct("Nop").finish(),
            Type::Class { name_index } => f
                .debug_struct("Class")
                .field("name_index", name_index)
                .field(
                    "name",
                    &self
                        .cp
                        .get(*name_index as usize)
                        .map(|t| t.as_cp_item(self.cp)),
                )
                .finish(),
            Type::Utf8 { bytes } => f
                .debug_struct("Utf8")
                .field("string", &std::str::from_utf8(bytes))
                .finish(),
            _ => write!(f, "TODO debug for: {:?}", self.item),
        }
    }
}
