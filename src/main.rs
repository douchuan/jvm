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

  x. fix warn & fixme

  x. verify class file
  x. java to execute a jar by -jar
  x. reimpl instruction by macro
  x. 如何不用事先初始化vector，就可以对任意位置赋值?
  x. 官方的测试用例
  x. build thread system
    去掉native 函数的jt参数
  x. impl Type annotations
  x. new Constructor with Exception and Type Annotations
  x. String.intern
  x. refact Exception处理方式
  x. native 调用，需要构造一个Frame吗？
  x. writeBytes实现完整

  x. UTF-8导致的问题
    java_lang_System::jvm_initProperties注释掉了"UTF-8"相关的内容

    xx. 按正常流程加载"sun/nio/cs/ext/ExtendedCharsets"，
        ExtendedCharsets会加载大量的内容，好像陷入死循环，还是需要多等一会？不确定

    xx. java_lang_Class::forName0暂且跳过"sun/nio/cs/ext/ExtendedCharsets"
      会导致抛出Charset.ExtendedProviderHolder.extendedProvider抛出ClassNotFoundException
      但目前，native异常的处理，还不正常，处理不了
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
    use crate::oop::OopDesc;
    use std::hash::{Hash, Hasher};

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

        let mut v = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 0];
        v[1..5].clone_from_slice(&[5, 4, 3, 2][..]);
        assert_eq!(v, vec![1, 5, 4, 3, 2, 6, 7, 8, 9, 0]);
    }

    #[test]
    fn t_arc() {
        use std::sync::{Arc, Mutex};
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

        let str1 = Vec::from("hello, world");
        let str1 = new_ref!(str1);
        let v1 = Arc::new(Mutex::new(Box::new(OopDesc::new_str(str1))));
        let v2 = v1.clone();
        assert!(Arc::ptr_eq(&v1, &v2));

        //raw arc eq
        let v1 = Arc::new(Mutex::new(Box::new(1000)));
        let v1_clone = v1.clone();
        let v1 = Arc::into_raw(v1) as i32;
        let v2 = Arc::into_raw(v1_clone) as i32;
        assert_eq!(v1, v2);

        //raw Arc not eq
        let v1 = Arc::new(Mutex::new(Box::new(1000)));
        let v2 = Arc::new(Mutex::new(Box::new(1000)));
        let v1 = Arc::into_raw(v1) as i32;
        let v2 = Arc::into_raw(v2) as i32;
        assert_ne!(v1, v2);

        //hash eq
        let s1 = String::from_utf8_lossy(b"abcde").to_string();
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        s1.hash(&mut hasher);
        let s1_hash = hasher.finish();
        let s2 = String::from_utf8_lossy(b"abcde").to_string();
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        s2.hash(&mut hasher);
        let s2_hash = hasher.finish();
        assert_eq!(s1_hash, s2_hash);

        //hash not eq
        let s1 = String::from_utf8_lossy(b"abcde").to_string();
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        s1.hash(&mut hasher);
        let s1_hash = hasher.finish();
        let s2 = String::from_utf8_lossy(b"abcde2").to_string();
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        s2.hash(&mut hasher);
        let s2_hash = hasher.finish();
        assert_ne!(s1_hash, s2_hash);

        //hash eq
        let s1 = "abcde".as_bytes();
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        s1.hash(&mut hasher);
        let s1_hash = hasher.finish();
        let s2 = String::from_utf8_lossy(b"abcde").to_string();
        let s2 = s2.as_bytes();
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        s2.hash(&mut hasher);
        let s2_hash = hasher.finish();
        assert_eq!(s1_hash, s2_hash);
    }

    #[test]
    fn t_libc() {
        let ptr = unsafe { libc::malloc(std::mem::size_of::<u8>() * 8) as *mut u8 };

        let l = 0x0102030405060708i64;
        let v = l.to_be_bytes();
        let v = vec![v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]];
        unsafe {
            libc::memcpy(
                ptr as *mut libc::c_void,
                v.as_ptr() as *const libc::c_void,
                8,
            );
        }

        let v = unsafe { *ptr } as u8;
        assert_eq!(v, 0x01u8);
    }
}
