use crate::oop::OopRef;

pub struct Exception {
    pub cls_name: &'static [u8],
    pub msg: Option<String>,
    pub ex_oop: Option<OopRef>,
}
