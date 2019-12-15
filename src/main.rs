extern crate bytes;
extern crate clap;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate env_logger;

use clap::{Arg, App};

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

  x. verify class file
  x. java to execute a jar by -jar
  x. try to opt by Ref with Lifetime
*/

fn init() {
   runtime::init();
}

fn main() {
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

    env_logger::init();
    init();

    let main_class = matches.value_of("MAIN_CLASS").unwrap();
    let args= matches.values_of("ARGS");
    println!("main class: {}, args: {:?}", main_class, args);

    let main = runtime::require_class(None, main_class).unwrap();
    let main_method = main.lock().unwrap().get_static_method("([Ljava/lang/String;)V", "main").unwrap();
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
