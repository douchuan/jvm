#[macro_use]
pub mod macros;

pub mod debug;
mod file;
mod net;
pub mod oop;
mod sync;
mod sys;

pub use self::file::*;
pub use self::net::*;
pub use self::sync::*;
pub use self::sys::*;

use crate::classfile::types::BytesRef;
pub fn new_method_id(name: &[u8], desc: &[u8]) -> BytesRef {
    let id = vec![name, desc].join(PATH_DELIMITER);
    new_ref!(id)
}

pub fn new_field_id(cls: &[u8], name: &[u8], desc: &[u8]) -> BytesRef {
    let id = vec![cls, name, desc].join(PATH_DELIMITER);
    new_ref!(id)
}
