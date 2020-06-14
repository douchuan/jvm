#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

//contains macros, must be here
#[macro_use]
pub mod util;

pub mod native;
pub mod oop;
pub mod runtime;

pub mod types;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

use classfile::BytesRef;
#[inline]
pub fn new_br(s: &str) -> BytesRef {
    std::sync::Arc::new(Vec::from(s))
}
