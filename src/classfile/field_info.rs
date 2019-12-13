use crate::classfile::attr_info::AttrType;
use crate::classfile::types::*;

#[derive(Debug)]
pub struct FieldInfo {
    pub acc_flags: U2,
    pub name_index: U2,
    pub desc_index: U2,
    pub attrs_count: U2,
    pub attrs: Vec<AttrType>,
}
