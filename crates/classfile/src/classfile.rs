use crate::attributes::InnerClass;
use crate::types::{ConstantPool, U2};
use crate::{
    attributes::Type, checker::CheckResult, checker::Checker, field_info::FieldInfo,
    method_info::MethodInfo, version::Version,
};

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

    pub fn inner_classes(&self) -> Option<Vec<InnerClass>> {
        for it in self.attrs.iter() {
            match it {
                Type::InnerClasses { classes } => {
                    return Some(classes.clone());
                }
                _ => (),
            }
        }

        None
    }

    pub fn signature(&self) -> Option<usize> {
        for it in self.attrs.iter() {
            match it {
                Type::Signature { signature_index } => {
                    return Some(*signature_index as usize);
                }
                _ => (),
            }
        }

        None
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
