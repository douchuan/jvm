use crate::classfile::attributes::Type;
use crate::types::U2;

#[derive(Debug)]
pub struct FieldInfo {
    pub acc_flags: U2,
    pub name_index: U2,
    pub desc_index: U2,
    pub attrs: Vec<Type>,
}
