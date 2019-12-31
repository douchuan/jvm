use crate::oop::OopDesc;
use std::sync::Arc;

pub enum Slot {
    ConstM1,
    Const0,
    Const1,
    Const2,
    Const3,
    Const4,
    Const5,
    Primitive(Vec<u8>),
    Utf8(Arc<Vec<u8>>),
    Ref(Arc<OopDesc>),
}
