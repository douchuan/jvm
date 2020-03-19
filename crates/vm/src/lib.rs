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

mod types;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
