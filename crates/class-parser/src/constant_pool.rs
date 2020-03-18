use crate::checker::CheckResult;
use crate::checker::{self, Checker};
use crate::consts::{
    CONSTANT_INTERFACE_METHOD_REF_TAG, CONSTANT_METHOD_REF_TAG, METHOD_NAME_CLINIT,
    METHOD_NAME_INIT,
};
use crate::signature::{MethodSignature, Type as SigType};
use crate::types::{BytesRef, ConstantPool};
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

impl Checker for Type {
    fn check(&self, cp: &ConstantPool) -> CheckResult {
        match self {
            Type::Nop => Ok(()),
            Type::Class { name_index } => match cp.get(*name_index as usize) {
                Some(Type::Utf8 { .. }) => Ok(()),
                _ => Err(checker::Err::InvalidCpClassNameIdx),
            },
            Type::FieldRef {
                class_index,
                name_and_type_index,
            } => {
                match cp.get(*class_index as usize) {
                    //todo: may be either a class type or an interface type
                    Some(Type::Class { name_index: _ }) => (),
                    _ => return Err(checker::Err::InvalidCpFieldRefClsIdx),
                }

                //todo:  the indicated descriptor must be a field descriptor (§4.3.2).
                // Otherwise, the indicated descriptor must be a method descriptor (§4.3.3).
                //If the name of the method of a CONSTANT_Methodref_info structure
                // begins with a '<' ('\u003c'), then the name must be the special
                // name <init>, representing an instance initialization method (§2.9).
                // The return type of such a method must be void
                match cp.get(*name_and_type_index as usize) {
                    Some(Type::NameAndType {
                        name_index,
                        desc_index,
                    }) => Ok(()),
                    _ => Err(checker::Err::InvalidCpFieldRefNameAndTypeIdx),
                }
            }
            Type::MethodRef {
                class_index,
                name_and_type_index,
            } => {
                //todo: must be a class type, not an interface type
                match cp.get(*class_index as usize) {
                    Some(Type::Class { name_index: _ }) => (),
                    _ => return Err(checker::Err::InvalidCpMethodRefClsIdx),
                }

                match get_name_and_type(cp, *name_and_type_index as usize) {
                    (Some(name), Some(desc)) => {
                        if name.starts_with(b"<") {
                            if name.as_slice() == METHOD_NAME_INIT {
                                let sig = MethodSignature::new(desc.as_slice());
                                if sig.retype == SigType::Void {
                                    Ok(())
                                } else {
                                    Err(checker::Err::InvalidCpMethodRefNameAndTypeIdx)
                                }
                            } else {
                                Err(checker::Err::InvalidCpMethodRefNameAndTypeIdx)
                            }
                        } else {
                            Ok(())
                        }
                    }
                    _ => Err(checker::Err::InvalidCpMethodRefNameAndTypeIdx),
                }
            }
            Type::InterfaceMethodRef {
                class_index,
                name_and_type_index,
            } => match cp.get(*class_index as usize) {
                Some(Type::Class { name_index: _ }) => Ok(()),
                _ => Err(checker::Err::InvalidCpInterfaceMethodRefClsIdx),
            },
            Type::String { string_index } => match cp.get(*string_index as usize) {
                Some(Type::Utf8 { .. }) => Ok(()),
                _ => Err(checker::Err::InvalidCpStrStrIdx),
            },
            Type::Integer { v: _ } => Ok(()),
            Type::Float { v: _ } => Ok(()),
            Type::Long { v: _ } => Ok(()),
            Type::Double { v: _ } => Ok(()),
            Type::NameAndType {
                name_index,
                desc_index,
            } => {
                match cp.get(*name_index as usize) {
                    Some(Type::Utf8 { .. }) => (),
                    _ => return Err(checker::Err::InvalidCpStrStrIdx),
                }

                match cp.get(*desc_index as usize) {
                    Some(Type::Utf8 { .. }) => Ok(()),
                    _ => return Err(checker::Err::InvalidCpStrStrIdx),
                }
            }
            Type::Utf8 { .. } => Ok(()),
            Type::MethodHandle {
                ref_kind,
                ref_index,
            } => {
                match ref_kind {
                    1..=9 => (),
                    _ => return Err(checker::Err::InvalidCpMethodHandleRefKind),
                }

                match ref_kind {
                    1 | 2 | 3 | 4 => match cp.get(*ref_index as usize) {
                        Some(Type::FieldRef {
                            class_index: _,
                            name_and_type_index: _,
                        }) => (),
                        _ => return Err(checker::Err::InvalidCpMethodHandleRefIdx),
                    },
                    5 | 8 => match cp.get(*ref_index as usize) {
                        Some(Type::MethodRef {
                            class_index: _,
                            name_and_type_index: _,
                        }) => (),
                        _ => return Err(checker::Err::InvalidCpMethodHandleRefIdx),
                    },
                    6 | 7 => match cp.get(*ref_index as usize) {
                        //fixme: is less than 52.0, the constant_pool entry at ...
                        Some(Type::MethodRef {
                            class_index: _,
                            name_and_type_index: _,
                        }) => (),
                        Some(Type::InterfaceMethodRef {
                            class_index,
                            name_and_type_index,
                        }) => (),
                        _ => return Err(checker::Err::InvalidCpMethodHandleRefIdx),
                    },
                    9 => match cp.get(*ref_index as usize) {
                        Some(Type::InterfaceMethodRef {
                            class_index: _,
                            name_and_type_index: _,
                        }) => (),
                        _ => return Err(checker::Err::InvalidCpMethodHandleRefIdx),
                    },
                    _ => return Err(checker::Err::InvalidCpMethodHandleRefIdx),
                }

                match ref_kind {
                    5 | 6 | 7 | 9 => match cp.get(*ref_index as usize) {
                        Some(Type::MethodRef {
                            class_index: _,
                            name_and_type_index,
                        }) => match get_name_and_type(cp, *name_and_type_index as usize) {
                            (Some(name), Some(desc)) => {
                                if name.as_slice() == METHOD_NAME_INIT
                                    || name.as_slice() == METHOD_NAME_CLINIT
                                {
                                    Err(checker::Err::InvalidCpMethodHandleRefIdx)
                                } else {
                                    Ok(())
                                }
                            }
                            _ => Err(checker::Err::InvalidCpMethodHandleRefIdx),
                        },
                        Some(Type::InterfaceMethodRef {
                            class_index: _,
                            name_and_type_index,
                        }) => match get_name_and_type(cp, *name_and_type_index as usize) {
                            (Some(name), Some(desc)) => {
                                if name.as_slice() == METHOD_NAME_INIT
                                    || name.as_slice() == METHOD_NAME_CLINIT
                                {
                                    Err(checker::Err::InvalidCpMethodHandleRefIdx)
                                } else {
                                    Ok(())
                                }
                            }
                            _ => Err(checker::Err::InvalidCpMethodHandleRefIdx),
                        },
                        _ => Err(checker::Err::InvalidCpMethodHandleRefIdx),
                    },
                    8 => match cp.get(*ref_index as usize) {
                        Some(Type::MethodRef {
                            class_index: _,
                            name_and_type_index,
                        }) => match get_name_and_type(cp, *name_and_type_index as usize) {
                            (Some(name), Some(desc)) if name.as_slice() == METHOD_NAME_INIT => {
                                Ok(())
                            }
                            _ => Err(checker::Err::InvalidCpMethodHandleRefIdx),
                        },
                        _ => Err(checker::Err::InvalidCpMethodHandleRefIdx),
                    },
                    _ => Err(checker::Err::InvalidCpMethodHandleRefIdx),
                }
            }
            Type::MethodType { desc_index } => match cp.get(*desc_index as usize) {
                Some(Type::Utf8 { .. }) => Ok(()),
                _ => Err(checker::Err::InvalidCpStrStrIdx),
            },
            Type::InvokeDynamic {
                bootstrap_method_attr_index,
                name_and_type_index,
            } => {
                //todo: bootstrap_method_attr_index
                match cp.get(*name_and_type_index as usize) {
                    Some(Type::NameAndType {
                        name_index: _,
                        desc_index: _,
                    }) => Ok(()),
                    _ => Err(checker::Err::InvalidCpFieldRefNameAndTypeIdx),
                }
            }
            Type::Unknown => Ok(()),
        }
    }
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
//
//impl AsRef<[u8]> for ConstantUtf8 {
//    fn as_ref(&self) -> &[u8] {
//        self.bytes.as_slice()
//    }
//}
