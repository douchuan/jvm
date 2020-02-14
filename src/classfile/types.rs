use crate::classfile::checker;
use crate::classfile::constant_pool::ConstantType;
use std::sync::Arc;

pub type U1 = u8;
pub type U2 = u16;
pub type U4 = u32;

pub type CheckResult = Result<(), checker::Err>;

//todo: rename to CharsRef
def_ref!(BytesRef, Vec<u8>);
def_ref!(ConstantPool, Vec<ConstantType>);
