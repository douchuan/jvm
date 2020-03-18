#![allow(unused)]

#[macro_use]
extern crate log;

pub mod attributes;
mod checker;
mod classfile;
pub mod constant_pool;
pub mod consts;
mod field_info;
pub mod flags;
mod method_info;
mod opcode;
mod parser;
mod signature;
pub mod types;
mod version;

pub use attributes::Type as AttributeType;
pub use classfile::ClassFile;
pub use constant_pool::Type as ConstantPoolType;
pub use field_info::FieldInfo;
pub use method_info::MethodInfo;
pub use opcode::OpCode;
pub use parser::parse as parse_class;
pub use signature::FieldSignature;
pub use signature::MethodSignature;
pub use signature::Type as SignatureType;
pub use types::BytesRef;
pub use types::ConstantPool;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
