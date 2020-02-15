use crate::types::{BytesRef, OopRef};

#[derive(Clone)]
pub struct Exception {
    pub cls_name: BytesRef,
    pub msg: Option<String>,
    pub ex_oop: Option<OopRef>,
}
