use crate::classfile::attr_info;
use crate::classfile::types::*;

#[derive(Debug, Clone)]
pub struct MethodInfo {
    pub acc_flags: U2,
    pub name_index: U2,
    pub desc_index: U2,
    attrs_count: U2,
    pub attrs: Vec<attr_info::AttrType>,
}

impl MethodInfo {
    pub fn new(
        acc_flags: U2,
        name_index: U2,
        desc_index: U2,
        attrs_count: U2,
        attrs: Vec<attr_info::AttrType>,
    ) -> Self {
        Self {
            acc_flags,
            name_index,
            desc_index,
            attrs_count,
            attrs,
        }
    }

    pub fn get_code(&self) -> &attr_info::Code {
        for it in self.attrs.iter() {
            match it {
                attr_info::AttrType::Code(code) => return code,
                _ => (),
            }
        }

        unreachable!()
    }
}
