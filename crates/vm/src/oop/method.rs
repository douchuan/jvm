use crate::oop::{self, ClassRef, ValueType};
use crate::runtime::{self, require_class2};
use crate::types::*;
use crate::util;
use crate::util::PATH_SEP;
use classfile::{
    attributes::Code, attributes::LineNumber, constant_pool, consts, flags::*, types::U2,
    AttributeType, BytesRef, ConstantPool, FieldInfo, MethodInfo,
};
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;
use std::ops::Deref;
use std::sync::Arc;

pub fn get_method_ref(cp: &ConstantPool, idx: usize) -> Result<MethodIdRef, ()> {
    let (tag, class_index, name_and_type_index) = constant_pool::get_method_ref(cp, idx);

    //load Method's Class, then init it
    let class = require_class2(class_index, cp).unwrap_or_else(|| {
        panic!(
            "Unknown method class {:?}",
            cp.get(class_index as usize)
                .expect("Missing item")
                .as_cp_item(cp)
        )
    });

    {
        let mut class = class.write().unwrap();
        class.init_class();
    }

    let (name, desc) = {
        let (name, typ) = constant_pool::get_name_and_type(cp, name_and_type_index as usize);
        let name = name.unwrap();
        let typ = typ.unwrap();

        (name, typ)
    };

    oop::class::init_class_fully(class.clone());

    let class = class.read().unwrap();

    trace!(
        "get_method_ref cls={}, name={}, desc={}",
        unsafe { std::str::from_utf8_unchecked(class.name.as_slice()) },
        unsafe { std::str::from_utf8_unchecked(name.as_slice()) },
        unsafe { std::str::from_utf8_unchecked(desc.as_slice()) },
    );

    let mir = if tag == consts::CONSTANT_METHOD_REF_TAG {
        // invokespecial, invokestatic and invokevirtual
        class.get_class_method(name, desc)
    } else {
        // invokeinterface
        class.get_interface_method(name, desc)
    };

    mir
}

#[derive(Debug, Clone)]
pub struct MethodId {
    pub offset: usize,
    pub method: Method,
}

#[derive(Clone)]
pub struct Method {
    pub class: ClassRef,
    pub class_file: ClassFileRef,
    pub cls_name: BytesRef,
    pub name: BytesRef,
    pub desc: BytesRef,
    pub acc_flags: U2,

    pub code: Option<Code>,
    pub line_num_table: Vec<LineNumber>,

    method_info_index: usize,
}

impl Method {
    pub fn new(
        cp: &ConstantPool,
        mi: &MethodInfo,
        class: ClassRef,
        class_file: ClassFileRef,
        method_info_index: usize,
        cls_name: BytesRef
    ) -> Self {
        let name = constant_pool::get_utf8(cp, mi.name_index as usize).unwrap();
        let desc = constant_pool::get_utf8(cp, mi.desc_index as usize).unwrap();
        let acc_flags = mi.acc_flags;
        let code = mi.get_code();
        let line_num_table = mi.get_line_number_table();

        Self {
            class,
            class_file,
            cls_name,
            name,
            desc,
            acc_flags,
            code,
            line_num_table,
            method_info_index,
        }
    }

    pub fn find_exception_handler(&self, cp: &ConstantPool, pc: U2, ex: ClassRef) -> Option<U2> {
        match &self.code {
            Some(code) => {
                for e in code.exceptions.iter() {
                    if e.contains(pc) {
                        if e.is_finally() {
                            return Some(e.handler_pc);
                        }

                        if let Some(class) = runtime::require_class2(e.catch_type, cp) {
                            if runtime::cmp::instance_of(ex.clone(), class) {
                                return Some(e.handler_pc);
                            }
                        }
                    }
                }
            }

            _ => (),
        }

        None
    }

    pub fn get_line_num(&self, pc: U2) -> i32 {
        let mut best_bci = 0;
        let mut best_line = -1;

        for it in self.line_num_table.iter() {
            if it.start_pc == pc {
                return it.number as i32;
            } else {
                if it.start_pc < pc && it.start_pc >= best_bci {
                    best_bci = it.start_pc;
                    best_line = it.number as i32;
                }
            }
        }

        return best_line;
    }

    pub fn get_annotation(&self) -> Option<Vec<u8>> {
        let method_info = self.class_file.methods.get(self.method_info_index).unwrap();
        util::attributes::assemble_annotation(&method_info.attrs)
    }

    pub fn get_param_annotation(&self) -> Option<Vec<u8>> {
        let method_info = self.class_file.methods.get(self.method_info_index).unwrap();
        util::attributes::assemble_param_annotation(&method_info.attrs)
    }

    pub fn get_type_annotation(&self) -> Option<Vec<u8>> {
        let method_info = self.class_file.methods.get(self.method_info_index).unwrap();
        util::attributes::assemble_type_annotation(&method_info.attrs)
    }

    pub fn get_annotation_default(&self) -> Option<Vec<u8>> {
        let method_info = self.class_file.methods.get(self.method_info_index).unwrap();
        util::attributes::assemble_annotation_default(&method_info.attrs)
    }

    pub fn check_annotation(&self, name: &[u8]) -> bool {
        let method_info = self.class_file.methods.get(self.method_info_index).unwrap();

        for it in method_info.attrs.iter() {
            match it {
                AttributeType::RuntimeVisibleAnnotations { raw, annotations } => {
                    for it in annotations.iter() {
                        if it.type_name.as_slice() == name {
                            return true;
                        }
                    }
                }
                _ => (),
            }
        }

        false
    }

    pub fn is_public(&self) -> bool {
        self.acc_flags & ACC_PUBLIC != 0
    }

    pub fn is_private(&self) -> bool {
        self.acc_flags & ACC_PRIVATE != 0
    }

    pub fn is_protected(&self) -> bool {
        self.acc_flags & ACC_PROTECTED != 0
    }

    pub fn is_final(&self) -> bool {
        self.acc_flags & ACC_FINAL != 0
    }

    pub fn is_static(&self) -> bool {
        self.acc_flags & ACC_STATIC != 0
    }

    pub fn is_synchronized(&self) -> bool {
        self.acc_flags & ACC_SYNCHRONIZED != 0
    }

    pub fn is_native(&self) -> bool {
        self.acc_flags & ACC_NATIVE != 0
    }

    pub fn is_abstract(&self) -> bool {
        self.acc_flags & ACC_ABSTRACT != 0
    }

    pub fn is_interface(&self) -> bool {
        self.acc_flags & ACC_INTERFACE != 0
    }
}

impl fmt::Debug for Method {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let cls_name = unsafe { std::str::from_utf8_unchecked(self.cls_name.as_slice()) };
        let name = unsafe { std::str::from_utf8_unchecked(self.name.as_slice()) };
        let desc = unsafe { std::str::from_utf8_unchecked(self.desc.as_slice()) };
        write!(f, "{}:{}:{}", cls_name, name, desc)
    }
}
