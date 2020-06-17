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

pub fn init_vm() {
    oop::init();
    runtime::init();
    native::init();
}

#[inline]
pub fn new_br(s: &str) -> classfile::BytesRef {
    std::sync::Arc::new(Vec::from(s))
}
