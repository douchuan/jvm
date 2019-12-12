extern crate bytes;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate env_logger;

#[macro_use]
mod oop;
mod classfile;
mod parser;
mod runtime;
mod util;

/*
todo:
  0. oop impl Oop，结合 runtime Slot，建立Ref(Oop)
    Oop String => Oop Object, rust实现java-lang-string
  1. runtime bytecode object 相关部分
  2. JNI

  x. verify class file
*/

fn main() {
    env_logger::init();
    classfile::init();

    //    parser::parse("test/Sample.class");
    //    parser::parse("test/Student.class");
    //    parser::parse("test/IPlayer.class");
    let path = "test/Test.class";
    match parser::parse(path) {
        Ok(c) => match c.check_format() {
            Ok(_) => println!("ok"),
            _ => error!("check format failed"),
        },
        _ => error!("class file parse failed"),
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn t_basic() {
        match 5 {
            1..=5 => assert!(true),
            _ => assert!(false),
        }
    }
}
