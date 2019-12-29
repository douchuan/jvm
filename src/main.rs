extern crate bytes;
extern crate clap;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate env_logger;

use crate::runtime::JavaMainThread;
use clap::{App, Arg};

#[macro_use]
mod oop;
mod classfile;
mod parser;
mod runtime;
mod util;

/*
todo list

  0. oop impl Oop，结合 runtime Slot，建立Ref(Oop)
    Oop String => Oop Object, 实现java-lang-string
  1. runtime byte code object 相关部分
  2. JNI
  3. runtime::execution::instance_of
  4. oop::class
    new_object_ary/new_prime_ary/new_wrapped_ary
  5. impl InstOopDesc
  6. impl runtime::thread::JavaThread run
  7. oop class init_class, java call "<clinit>"
     impl JavaCall::with_args(thread, clinit, {})
  8. impl byte code instruction

  x. throw exception, aload...
  x. verify class file
  x. java to execute a jar by -jar
*/

fn init_vm() {
    runtime::init();
}

fn main() {
    env_logger::init();

    let matches = App::new("")
        .arg(
            Arg::with_name("cp")
                .long("cp")
                .help("class search path of directories and zip/jar files")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("classpath")
                .long("classpath")
                .help("class search path of directories and zip/jar files")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("MAIN_CLASS")
                .help("to execute a class")
                .required(true)
                .index(1),
        )
        .arg(Arg::with_name("ARGS").multiple(true).help("[args...]"))
        .get_matches();

    let main_class = matches.value_of_lossy("MAIN_CLASS").unwrap().to_string();
    let args = matches.values_of_lossy("ARGS");
    println!("main class: {}, args: {:?}", main_class, args);

    init_vm();

    let thread = JavaMainThread { main_class, args };
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

        let s1: &[u8] = b"12345";
        let s2: &[u8] = b"67890";
        let s3: &[u8] = b"abcde";
        let sep: &[u8] = b":";
        assert_eq!(vec![s1, s2, s3].join(sep), b"12345:67890:abcde");
    }

    #[test]
    fn t_arc() {
        use std::sync::Arc;
        struct TestArc {
            bytes: Arc<Vec<u8>>,
        }

        let mut ref_bytes = None;
        {
            let bytes = Arc::new(vec![1, 2, 3, 4]);
            let t = TestArc { bytes };
            ref_bytes = Some(t.bytes.clone());
            assert_eq!(2, Arc::strong_count(&t.bytes));
        }
        assert!(ref_bytes.is_some());
        assert_eq!(ref_bytes, Some(Arc::new(vec![1, 2, 3, 4])));
        assert_eq!(1, Arc::strong_count(&ref_bytes.unwrap()));
    }
}
