use crate::classfile::attr_info::AttrType;
use crate::classfile::types::*;

#[derive(Debug)]
pub struct FieldInfo {
    pub acc_flags: U2,
    pub name_index: U2,
    pub desc_index: U2,
    attrs_count: U2,
    pub attrs: Vec<AttrType>,
}

impl FieldInfo {
    pub fn new(
        acc_flags: U2,
        name_index: U2,
        desc_index: U2,
        attrs_count: U2,
        attrs: Vec<AttrType>,
    ) -> Self {
        Self {
            acc_flags,
            name_index,
            desc_index,
            attrs_count,
            attrs,
        }
    }
}
