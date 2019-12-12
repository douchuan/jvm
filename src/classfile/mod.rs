#![allow(unused)]

pub mod access_flags;
pub mod attr_info;
pub mod checker;
pub mod constant_pool;
pub mod consts;
pub mod field_info;
pub mod method_info;
pub mod opcode;
pub mod signature;
pub mod types;

pub use crate::classfile::attr_info::AttrType;
use crate::classfile::checker::Checker;
pub use crate::classfile::field_info::FieldInfo;
pub use crate::classfile::method_info::MethodInfo;
use crate::classfile::types::*;
use crate::runtime;

pub fn init() {
    runtime::system_dictionary::init();
    runtime::class_path_manager::init();
}

#[derive(Debug)]
pub struct ClassFile {
    magic: U4,
    version: Version,
    cp_count: U2,
    pub cp: ConstantPool,
    pub acc_flags: U2,
    pub this_class: U2,
    pub super_class: U2,
    interfaces_count: U2,
    pub interfaces: Vec<U2>,
    pub fields_count: U2,
    pub fields: Vec<FieldInfo>,
    methods_count: U2,
    pub methods: Vec<MethodInfo>,
    attrs_count: U2,
    pub attrs: Vec<AttrType>,
}

impl ClassFile {
    pub fn new(
        magic: U4,
        version: Version,
        cp_count: U2,
        cp: ConstantPool,
        acc_flags: U2,
        this_class: U2,
        super_class: U2,
        interfaces_count: U2,
        interfaces: Vec<U2>,
        fields_count: U2,
        fields: Vec<FieldInfo>,
        methods_count: U2,
        methods: Vec<MethodInfo>,
        attrs_count: U2,
        attrs: Vec<AttrType>,
    ) -> Self {
        Self {
            magic,
            version,
            cp_count,
            cp,
            acc_flags,
            this_class,
            super_class,
            interfaces_count,
            interfaces,
            fields_count,
            fields,
            methods_count,
            methods,
            attrs_count,
            attrs,
        }
    }

    pub fn check_format(&self) -> CheckResult {
        self.check(&self.cp)
    }
}

#[derive(Debug)]
pub struct Version {
    minor: U2,
    major: U2,
}

impl Version {
    pub fn new(minor: U2, major: U2) -> Self {
        Self { minor, major }
    }
}

impl Checker for ClassFile {
    fn check(&self, cp: &ConstantPool) -> CheckResult {
        if self.magic != consts::MAGIC {
            Err(checker::Err::InvalidMagicNum)
        } else {
            for it in self.cp.as_slice() {
                let r = it.check(cp);
                if r.is_err() {
                    return r;
                }
            }

            Ok(())
        }
    }
}

//todo: impl me
fn is_valid_field_name() -> bool {
    true
}

fn is_valid_class_name() -> bool {
    true
}
