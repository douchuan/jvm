use crate::classfile::checker;
use crate::classfile::constant_pool::ConstantType;
use std::sync::Arc;

pub type U1 = u8;
pub type U2 = u16;
pub type U4 = u32;
pub type BytesRef = Arc<Vec<U1>>;

pub type ConstantPool = Vec<ConstantType>;
pub type CheckResult = Result<(), checker::Err>;
