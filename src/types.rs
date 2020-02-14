use crate::classfile::constant_pool::ConstantType;

pub type U1 = u8;
pub type U2 = u16;
pub type U4 = u32;

//todo: rename to CharsRef
def_ref!(BytesRef, Vec<u8>);
def_ref!(ConstantPool, Vec<ConstantType>);
