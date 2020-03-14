use crate::oop::{class, Oop};
use crate::types::*;

#[derive(Debug, Clone)]
pub struct ArrayOopDesc {
    pub class: ClassRef,
    pub elements: Vec<Oop>,
}

#[derive(Debug, Clone)]
pub enum TypeArrayDesc {
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

impl TypeArrayDesc {
    pub fn len(&self) -> usize {
        match self {
            TypeArrayDesc::Char(ary) => ary.len(),
            TypeArrayDesc::Byte(ary) => ary.len(),
            TypeArrayDesc::Bool(ary) => ary.len(),
            TypeArrayDesc::Short(ary) => ary.len(),
            TypeArrayDesc::Float(ary) => ary.len(),
            TypeArrayDesc::Double(ary) => ary.len(),
            TypeArrayDesc::Int(ary) => ary.len(),
            TypeArrayDesc::Long(ary) => ary.len(),
        }
    }
}
