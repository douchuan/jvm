use crate::classfile::{
    attributes::Type, checker::CheckResult, checker::Checker, field_info::FieldInfo,
    method_info::MethodInfo, Version,
};
use crate::types::{ConstantPool, U2};

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
