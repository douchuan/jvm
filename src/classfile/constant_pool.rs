use crate::classfile::checker::{self, Checker};
use crate::classfile::consts::{
    CONSTANT_INTERFACE_METHOD_REF_TAG, CONSTANT_METHOD_REF_TAG, METHOD_NAME_CLINIT,
    METHOD_NAME_INIT,
};
use crate::classfile::signature::{MethodSignature, Type as SigType};
use crate::classfile::types::{BytesRef, CheckResult, ConstantPool};
use std::fmt;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum ConstantType {
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
        length: u16,
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

pub fn get_class_name(cp: &ConstantPool, idx: usize) -> Option<BytesRef> {
    match cp.get(idx) {
        Some(ConstantType::Class { name_index }) => get_utf8(cp, *name_index as usize),
        _ => None,
    }
}

pub fn get_field_ref(cp: &ConstantPool, idx: usize) -> (u16, u16) {
    match cp.get(idx) {
        Some(ConstantType::FieldRef {
            class_index,
            name_and_type_index,
        }) => (*class_index, *name_and_type_index),
        _ => unreachable!(),
    }
}

pub fn get_method_ref(cp: &ConstantPool, idx: usize) -> (u8, u16, u16) {
    match cp.get(idx) {
        Some(ConstantType::MethodRef {
            class_index,
            name_and_type_index,
        }) => (CONSTANT_METHOD_REF_TAG, *class_index, *name_and_type_index),
        Some(ConstantType::InterfaceMethodRef {
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
        Some(ConstantType::NameAndType {
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
        Some(ConstantType::Utf8 { length: _, bytes }) => Some(bytes.clone()),
        _ => None,
    }
}

impl Checker for ConstantType {
    fn check(&self, cp: &ConstantPool) -> CheckResult {
        match self {
            ConstantType::Nop => Ok(()),
            ConstantType::Class { name_index } => match cp.get(*name_index as usize) {
                Some(ConstantType::Utf8 {
                    length: _,
                    bytes: _,
                }) => Ok(()),
                _ => Err(checker::Err::InvalidCpClassNameIdx),
            },
            ConstantType::FieldRef {
                class_index,
                name_and_type_index,
            } => {
                match cp.get(*class_index as usize) {
                    //todo: may be either a class type or an interface type
                    Some(ConstantType::Class { name_index: _ }) => (),
                    _ => return Err(checker::Err::InvalidCpFieldRefClsIdx),
                }

                //todo:  the indicated descriptor must be a field descriptor (§4.3.2).
                // Otherwise, the indicated descriptor must be a method descriptor (§4.3.3).
                //If the name of the method of a CONSTANT_Methodref_info structure
                // begins with a '<' ('\u003c'), then the name must be the special
                // name <init>, representing an instance initialization method (§2.9).
                // The return type of such a method must be void
                match cp.get(*name_and_type_index as usize) {
                    Some(ConstantType::NameAndType {
                        name_index,
                        desc_index,
                    }) => Ok(()),
                    _ => Err(checker::Err::InvalidCpFieldRefNameAndTypeIdx),
                }
            }
            ConstantType::MethodRef {
                class_index,
                name_and_type_index,
            } => {
                //todo: must be a class type, not an interface type
                match cp.get(*class_index as usize) {
                    Some(ConstantType::Class { name_index: _ }) => (),
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
            ConstantType::InterfaceMethodRef {
                class_index,
                name_and_type_index,
            } => match cp.get(*class_index as usize) {
                Some(ConstantType::Class { name_index: _ }) => Ok(()),
                _ => Err(checker::Err::InvalidCpInterfaceMethodRefClsIdx),
            },
            ConstantType::String { string_index } => match cp.get(*string_index as usize) {
                Some(ConstantType::Utf8 {
                    length: _,
                    bytes: _,
                }) => Ok(()),
                _ => Err(checker::Err::InvalidCpStrStrIdx),
            },
            ConstantType::Integer { v: _ } => Ok(()),
            ConstantType::Float { v: _ } => Ok(()),
            ConstantType::Long { v: _ } => Ok(()),
            ConstantType::Double { v: _ } => Ok(()),
            ConstantType::NameAndType {
                name_index,
                desc_index,
            } => {
                match cp.get(*name_index as usize) {
                    Some(ConstantType::Utf8 {
                        length: _,
                        bytes: _,
                    }) => (),
                    _ => return Err(checker::Err::InvalidCpStrStrIdx),
                }

                match cp.get(*desc_index as usize) {
                    Some(ConstantType::Utf8 { length, bytes }) => Ok(()),
                    _ => return Err(checker::Err::InvalidCpStrStrIdx),
                }
            }
            ConstantType::Utf8 {
                length: _,
                bytes: _,
            } => Ok(()),
            ConstantType::MethodHandle {
                ref_kind,
                ref_index,
            } => {
                match ref_kind {
                    1..=9 => (),
                    _ => return Err(checker::Err::InvalidCpMethodHandleRefKind),
                }

                match ref_kind {
                    1 | 2 | 3 | 4 => match cp.get(*ref_index as usize) {
                        Some(ConstantType::FieldRef {
                            class_index: _,
                            name_and_type_index: _,
                        }) => (),
                        _ => return Err(checker::Err::InvalidCpMethodHandleRefIdx),
                    },
                    5 | 8 => match cp.get(*ref_index as usize) {
                        Some(ConstantType::MethodRef {
                            class_index: _,
                            name_and_type_index: _,
                        }) => (),
                        _ => return Err(checker::Err::InvalidCpMethodHandleRefIdx),
                    },
                    6 | 7 => match cp.get(*ref_index as usize) {
                        //fixme: is less than 52.0, the constant_pool entry at ...
                        Some(ConstantType::MethodRef {
                            class_index: _,
                            name_and_type_index: _,
                        }) => (),
                        Some(ConstantType::InterfaceMethodRef {
                            class_index,
                            name_and_type_index,
                        }) => (),
                        _ => return Err(checker::Err::InvalidCpMethodHandleRefIdx),
                    },
                    9 => match cp.get(*ref_index as usize) {
                        Some(ConstantType::InterfaceMethodRef {
                            class_index: _,
                            name_and_type_index: _,
                        }) => (),
                        _ => return Err(checker::Err::InvalidCpMethodHandleRefIdx),
                    },
                    _ => return Err(checker::Err::InvalidCpMethodHandleRefIdx),
                }

                match ref_kind {
                    5 | 6 | 7 | 9 => match cp.get(*ref_index as usize) {
                        Some(ConstantType::MethodRef {
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
                        Some(ConstantType::InterfaceMethodRef {
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
                        Some(ConstantType::MethodRef {
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
            ConstantType::MethodType { desc_index } => match cp.get(*desc_index as usize) {
                Some(ConstantType::Utf8 {
                    length: _,
                    bytes: _,
                }) => Ok(()),
                _ => Err(checker::Err::InvalidCpStrStrIdx),
            },
            ConstantType::InvokeDynamic {
                bootstrap_method_attr_index,
                name_and_type_index,
            } => {
                //todo: bootstrap_method_attr_index
                match cp.get(*name_and_type_index as usize) {
                    Some(ConstantType::NameAndType {
                        name_index: _,
                        desc_index: _,
                    }) => Ok(()),
                    _ => Err(checker::Err::InvalidCpFieldRefNameAndTypeIdx),
                }
            }
            ConstantType::Unknown => Ok(()),
        }
    }
}

#[derive(Debug)]
pub enum ConstantTag {
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
    Unknown,
}

impl From<u8> for ConstantTag {
    fn from(tag: u8) -> Self {
        match tag {
            7 => ConstantTag::Class,
            9 => ConstantTag::FieldRef,
            10 => ConstantTag::MethodRef,
            11 => ConstantTag::InterfaceMethodRef,
            8 => ConstantTag::String,
            3 => ConstantTag::Integer,
            4 => ConstantTag::Float,
            5 => ConstantTag::Long,
            6 => ConstantTag::Double,
            12 => ConstantTag::NameAndType,
            1 => ConstantTag::Utf8,
            15 => ConstantTag::MethodHandle,
            16 => ConstantTag::MethodType,
            18 => ConstantTag::InvokeDynamic,
            _ => ConstantTag::Unknown,
        }
    }
}

//impl fmt::Debug for ConstantUtf8 {
//    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//        write!(
//            f,
//            "ConstantUtf8Info(\"{}\", {})",
//            String::from_utf8_lossy(&self.bytes[0..self.length as usize]),
//            self.length
//        )
//    }
//}
//
//impl AsRef<[u8]> for ConstantUtf8 {
//    fn as_ref(&self) -> &[u8] {
//        self.bytes.as_slice()
//    }
//}
