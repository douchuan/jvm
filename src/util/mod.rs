#[macro_use]
pub mod macros;

pub mod debug;
mod file;
mod net;
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

pub fn new_field_id(cls_name: &[u8], name: &[u8], desc: &[u8]) -> BytesRef {
    let id = vec![cls_name, name, desc].join(PATH_DELIMITER);
    new_ref!(id)
}
