extern crate bytes;
extern crate clap;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate env_logger;

use clap::{App, Arg};

#[macro_use]
mod util;

mod classfile;
mod native;
mod oop;
mod parser;
mod runtime;

use crate::runtime::thread::JavaMainThread;

/*
todo list

  x. verify class file
  x. java to execute a jar by -jar
  x. reimpl instruction by macro
  x. 如何不用事先初始化vector，就可以对任意位置赋值?
  x. 官方的测试用例
  x. build thread system
    去掉native 函数的jt参数
  x. class mirror
    是底层class在上层的一个实例吗？为了与Oop统一管理
    classmirror field_value 个数和类型是什么？
  x. impl Type annotations
*/

fn init_vm() {
    oop::init();
    runtime::init();
    native::init();
}

fn main() {
    env_logger::init();
    init_vm();

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

    //todo: add '.' auto
    let cp = matches.value_of("cp");
    if let Some(cp) = cp {
        runtime::add_class_paths(cp);
    }

    let classpath = matches.value_of("classpath");
    if let Some(classpath) = classpath {
        runtime::add_class_path(classpath);
    }

    let class = matches.value_of_lossy("MAIN_CLASS").unwrap().to_string();
    let args = matches.values_of_lossy("ARGS");
    println!("main class: {}, args: {:?}", class, args);

    let thread = JavaMainThread { class, args };
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

        let mut v = Vec::new();
        v.insert(0, "aaa");
        v.push("bbb");

        assert_eq!(v[0], "aaa");

        let mut v = Vec::with_capacity(10);
        unsafe {
            v.set_len(10);
        }
        v[9] = 9;
        println!("v[0] = {}", v[0]);
        println!("v[9] = {}", v[9]);
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

        use crate::oop::Oop;
        use crate::runtime::Slot;
        let null1 = Arc::new(Oop::Null);
        let null2 = Arc::new(Oop::Null);
        assert!(!Arc::ptr_eq(&null1, &null2));
        let null11 = null1.clone();
        assert!(Arc::ptr_eq(&null1, &null1));
    }
}
