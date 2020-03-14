use crate::oop::{Oop, ValueType};
use crate::types::ClassRef;

#[derive(Debug, Clone)]
pub struct MirrorOopDesc {
    pub target: Option<ClassRef>,
    pub field_values: Vec<Oop>,
    pub value_type: ValueType,
}

impl MirrorOopDesc {
    pub fn is_prim_mirror(&self) -> bool {
        self.target.is_none()
    }
}
