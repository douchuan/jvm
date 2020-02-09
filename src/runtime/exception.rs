use crate::classfile::types::BytesRef;
use crate::oop::OopRef;

#[derive(Clone)]
pub struct Exception {
    pub cls_name: BytesRef,
    pub msg: Option<String>,
    pub ex_oop: Option<OopRef>,
}
