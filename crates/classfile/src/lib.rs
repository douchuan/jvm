#![allow(unused)]

//! Provides types for working with class file.
//!
//! The `classfile` crate provides types for describing the
//! class file format of the Java Virtual Machine.
//!
//! It's not class file parser.

#[macro_use]
extern crate log;
pub mod attributes;
mod classfile;
pub mod constant_pool;
pub mod consts;
mod field_info;
pub mod flags;
mod method_info;
mod opcode;
mod signature;
pub mod types;
mod version;

pub use crate::classfile::ClassFile;
pub use attributes::Type as AttributeType;
pub use constant_pool::Type as ConstantPoolType;
pub use field_info::FieldInfo;
pub use method_info::MethodInfo;
pub use opcode::OpCode;
pub use signature::Type as SignatureType;
pub use types::BytesRef;
pub use types::ConstantPool;
pub use version::Version;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
