#[macro_use]
pub mod macros;

pub mod oop;
mod sys;

pub use self::sys::*;

use crate::types::BytesRef;
use std::sync::Arc;

pub fn new_method_id(name: &[u8], desc: &[u8]) -> BytesRef {
    Arc::new(vec![name, desc].join(PATH_SEP.as_bytes()))
}

pub fn new_field_id(cls: &[u8], name: &[u8], desc: &[u8]) -> BytesRef {
    Arc::new(vec![cls, name, desc].join(PATH_SEP.as_bytes()))
}
