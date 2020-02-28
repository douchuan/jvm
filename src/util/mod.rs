#[macro_use]
pub mod macros;

pub mod oop;
mod sys;

pub use self::sys::*;

use crate::types::BytesRef;

pub fn new_method_id(name: &[u8], desc: &[u8]) -> BytesRef {
    let id = vec![name, desc].join(PATH_SEP.as_bytes());
    new_ref!(id)
}

pub fn new_field_id(cls: &[u8], name: &[u8], desc: &[u8]) -> BytesRef {
    let id = vec![cls, name, desc].join(PATH_SEP.as_bytes());
    new_ref!(id)
}
