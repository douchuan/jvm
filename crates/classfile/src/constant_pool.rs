use crate::consts::{
    CONSTANT_INTERFACE_METHOD_REF_TAG, CONSTANT_METHOD_REF_TAG, METHOD_NAME_CLINIT,
    METHOD_NAME_INIT,
};
use crate::{BytesRef, ConstantPool};
use fmt::Debug;
use std::fmt;
use std::sync::Arc;

pub fn get_class_name(cp: &ConstantPool, idx: usize) -> &BytesRef {
    match cp.get(idx) {
        Some(Type::Class { name_index }) => get_utf8(cp, *name_index as usize),
        _ => unreachable!(),
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

pub fn get_name_and_type(cp: &ConstantPool, idx: usize) -> (&BytesRef, &BytesRef) {
    match cp.get(idx) {
        Some(Type::NameAndType {
            name_index,
            desc_index,
        }) => (
            get_utf8(cp, *name_index as usize),
            get_utf8(cp, *desc_index as usize),
        ),
        _ => unreachable!(),
    }
}

pub fn get_utf8(cp: &ConstantPool, idx: usize) -> &BytesRef {
    match cp.get(idx) {
        Some(Type::Utf8 { bytes }) => bytes,
        _ => unreachable!(),
    }
}

pub fn get_string(cp: &ConstantPool, idx: usize) -> String {
    match cp.get(idx) {
        Some(Type::String { string_index }) => {
            let v = get_utf8(cp, *string_index as usize);
            let raw = construct_string_raw(v.as_slice());
            String::from_utf16_lossy(raw.as_slice())
        }
        _ => unreachable!(),
    }
}

pub fn construct_string_raw(bs: &[u8]) -> Vec<u16> {
    let length = bs.len();
    let mut buffer: Vec<u16> = Vec::with_capacity(length);
    let mut pos = 0;
    while pos < length {
        if bs[pos] & 0x80 == 0 {
            let v = bs[pos] as u16;
            buffer.push(v);
            pos += 1;
        } else if bs[pos] & 0xE0 == 0xC0 && (bs[pos + 1] & 0xC0) == 0x80 {
            let x = bs[pos] as u16;
            let y = bs[pos + 1] as u16;
            let v = ((x & 0x1f) << 6) + (y & 0x3f);
            buffer.push(v);
            pos += 2;
        } else if bs[pos] & 0xF0 == 0xE0
            && (bs[pos + 1] & 0xC0) == 0x80
            && (bs[pos + 2] & 0xC0) == 0x80
        {
            let x = bs[pos] as u16;
            let y = bs[pos + 1] as u16;
            let z = bs[pos + 2] as u16;
            let v = ((x & 0xf) << 12) + ((y & 0x3f) << 6) + (z & 0x3f);
            buffer.push(v);
            pos += 3;
        } else if bs[pos] == 0xED
            && (bs[pos + 1] & 0xF0 == 0xA0)
            && (bs[pos + 2] & 0xC0 == 0x80)
            && (bs[pos + 3] == 0xED)
            && (bs[pos + 4] & 0xF0 == 0xB0)
            && (bs[pos + 5] & 0xC0 == 0x80)
        {
            let v = bs[pos + 1] as u32;
            let w = bs[pos + 2] as u32;
            let y = bs[pos + 4] as u32;
            let z = bs[pos + 5] as u32;
            let vv =
                0x10000 + ((v & 0x0f) << 16) + ((w & 0x3f) << 10) + ((y & 0x0f) << 6) + (z & 0x3f);
            buffer.push(vv as u16);

            pos += 6;
        } else {
            unreachable!()
        }
    }

    buffer
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
