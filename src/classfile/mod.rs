#![allow(unused)]

pub mod attributes;
pub mod checker;
pub mod constant_pool;
pub mod consts;
pub mod field_info;
pub mod flags;
pub mod method_info;
pub mod opcode;
pub mod signature;

pub use crate::classfile::attributes::Type;
use crate::classfile::checker::{CheckResult, Checker};
pub use crate::classfile::field_info::FieldInfo;
pub use crate::classfile::method_info::MethodInfo;
use crate::types::*;

#[derive(Debug)]
pub struct ClassFile {
    pub version: Version,
    pub cp: ConstantPool,
    pub acc_flags: U2,
    pub this_class: U2,
    pub super_class: U2,
    pub interfaces: Vec<U2>,
    pub fields: Vec<FieldInfo>,
    pub methods: Vec<MethodInfo>,
    pub attrs: Vec<Type>,
}

impl ClassFile {
    pub fn check_format(&self) -> CheckResult {
        self.check(&self.cp)
    }
}

#[derive(Debug)]
pub struct Version {
    pub minor: U2,
    pub major: U2,
}

impl Checker for ClassFile {
    fn check(&self, cp: &ConstantPool) -> CheckResult {
        for it in self.cp.as_slice() {
            let r = it.check(cp);
            if r.is_err() {
                return r;
            }
        }

        Ok(())
    }
}
