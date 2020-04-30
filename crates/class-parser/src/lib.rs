#![allow(unused)]

mod class;
mod signature;

pub use class::parse as parse_class;
pub use signature::{ClassSignature, FieldSignature, MethodSignature};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
