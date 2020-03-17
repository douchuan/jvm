use crate::constant_pool;
use std::sync::Arc;

pub type U1 = u8;
pub type U2 = u16;
pub type U4 = u32;
pub type BytesRef = Arc<Vec<u8>>;
pub type ConstantPool = Arc<Vec<constant_pool::Type>>;
