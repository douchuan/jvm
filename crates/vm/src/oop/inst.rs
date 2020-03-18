use crate::oop::{field, Oop};
use crate::types::ClassRef;

#[derive(Debug, Clone)]
pub struct InstOopDesc {
    pub class: ClassRef,
    pub field_values: Vec<Oop>,
}

impl InstOopDesc {
    pub fn new(class: ClassRef) -> Self {
        let field_values = field::build_inited_field_values(class.clone());

        Self {
            class,
            field_values,
        }
    }
}
