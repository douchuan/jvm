use crate::oop::{class, Oop};
use crate::types::*;

#[derive(Debug, Clone)]
pub struct ArrayOopDesc {
    pub class: ClassRef,
    pub elements: Vec<Oop>,
}

#[derive(Debug, Clone)]
pub enum TypeArrayValue {
    Byte(ByteAry),
    Bool(BoolAry),
    Char(CharAry),
    Short(ShortAry),
    Float(FloatAry),
    Double(DoubleAry),
    Int(IntAry),
    Long(LongAry),
}
impl ArrayOopDesc {
    pub fn new(class: ClassRef, elements: Vec<Oop>) -> Self {
        {
            assert!(class.read().unwrap().is_array());
        }

        Self { class, elements }
    }

    pub fn get_dimension(&self) -> usize {
        let class = self.class.read().unwrap();
        match &class.kind {
            class::ClassKind::ObjectArray(ary_class_obj) => ary_class_obj.get_dimension(),
            class::ClassKind::TypeArray(ary_class_obj) => ary_class_obj.get_dimension(),
            _ => unreachable!(),
        }
    }
}

impl TypeArrayValue {
    pub fn len(&self) -> usize {
        match self {
            TypeArrayValue::Char(ary) => ary.len(),
            TypeArrayValue::Byte(ary) => ary.len(),
            TypeArrayValue::Bool(ary) => ary.len(),
            TypeArrayValue::Short(ary) => ary.len(),
            TypeArrayValue::Float(ary) => ary.len(),
            TypeArrayValue::Double(ary) => ary.len(),
            TypeArrayValue::Int(ary) => ary.len(),
            TypeArrayValue::Long(ary) => ary.len(),
        }
    }
}
