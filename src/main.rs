extern crate bytes;
extern crate clap;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate env_logger;

use clap::{Arg, App};
use crate::runtime::JavaMainThread;

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
  3. rutime::execution::instance_of
  4. oop::class
    new_object_ary/new_prime_ary/new_wrapped_ary
  5. impl InstOopDesc
  6. impl runtime::thread::JavaThread run

  x. verify class file
  x. java to execute a jar by -jar
  x. try to opt by Ref with Lifetime
*/

fn init_vm() {
   runtime::init();
}

fn main() {
    env_logger::init();

    let matches = App::new("")
        .arg(Arg::with_name("cp")
            .long("cp")
            .help("class search path of directories and zip/jar files")
            .takes_value(true))
        .arg(Arg::with_name("classpath")
            .long("classpath")
            .help("class search path of directories and zip/jar files")
            .takes_value(true))
        .arg(Arg::with_name("MAIN_CLASS")
            .help("to execute a class")
            .required(true)
            .index(1))
        .arg(Arg::with_name("ARGS")
            .multiple(true)
            .help("[args...]"))
        .get_matches();

    let main_class = matches.value_of_lossy("MAIN_CLASS").unwrap().to_string();
    let args= matches.values_of_lossy("ARGS");
    println!("main class: {}, args: {:?}", main_class, args);

    init_vm();

    let thread = JavaMainThread {main_class, args};
    thread.run();

    /*
    let path = "test/Test.class";
    match parser::parse(path) {
        Ok(c) => match c.check_format() {
            Ok(_) => println!("ok"),
            _ => error!("check format failed"),
        },
        _ => error!("class file parse failed"),
    }
    */
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
