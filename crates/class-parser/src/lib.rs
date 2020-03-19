#![allow(unused)]

mod parser;

pub use parser::parse as parse_class;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
