use crate::oop::OopRef;
use crate::types::BytesRef;

#[derive(Clone)]
pub struct Exception {
    pub cls_name: BytesRef,
    pub msg: Option<String>,
    pub ex_oop: Option<OopRef>,
}
