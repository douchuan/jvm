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

impl TypeArrayDesc {
    pub fn extract_chars(&self) -> &CharAry {
        match self {
            TypeArrayDesc::Char(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn extract_mut_chars(&mut self) -> &mut CharAry {
        match self {
            TypeArrayDesc::Char(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn extract_bytes(&self) -> &ByteAry {
        match self {
            TypeArrayDesc::Byte(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn extract_mut_bytes(&mut self) -> &mut ByteAry {
        match self {
            TypeArrayDesc::Byte(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn extract_bools(&self) -> &BoolAry {
        match self {
            TypeArrayDesc::Bool(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn extract_mut_bools(&mut self) -> &mut BoolAry {
        match self {
            TypeArrayDesc::Bool(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn extract_shorts(&self) -> &ShortAry {
        match self {
            TypeArrayDesc::Short(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn extract_mut_shorts(&mut self) -> &mut ShortAry {
        match self {
            TypeArrayDesc::Short(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn extract_floats(&self) -> &FloatAry {
        match self {
            TypeArrayDesc::Float(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn extract_mut_floats(&mut self) -> &mut FloatAry {
        match self {
            TypeArrayDesc::Float(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn extract_doubles(&self) -> &DoubleAry {
        match self {
            TypeArrayDesc::Double(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn extract_mut_doubles(&mut self) -> &mut DoubleAry {
        match self {
            TypeArrayDesc::Double(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn extract_ints(&self) -> &IntAry {
        match self {
            TypeArrayDesc::Int(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn extract_mut_ints(&mut self) -> &mut IntAry {
        match self {
            TypeArrayDesc::Int(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn extract_longs(&self) -> &LongAry {
        match self {
            TypeArrayDesc::Long(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn extract_mut_longs(&mut self) -> &mut LongAry {
        match self {
            TypeArrayDesc::Long(v) => v,
            _ => unreachable!(),
        }
    }
}
