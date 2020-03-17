#![allow(unused)]

#[macro_use]
extern crate log;

pub mod attributes;
pub mod checker;
pub mod classfile;
pub mod constant_pool;
pub mod consts;
pub mod field_info;
pub mod flags;
pub mod method_info;
pub mod opcode;
pub mod parser;
pub mod signature;
pub mod types;
pub mod version;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
