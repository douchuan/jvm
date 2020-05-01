extern crate clap;
extern crate env_logger;

use clap::{App, Arg};

use vm::native;
use vm::oop;
use vm::runtime;
use vm::runtime::thread::JavaMainThread;
use vm::util;

/*
todo list

  x. fix warn & fixme

  x. java to execute a jar by -jar
  x. Official test case, TCK
  x. build thread system
    Remove the jt parameter of the native function
  x. new Constructor with Exception
  x. String.intern
  x. What's the use of BootstrapMethod?
  x. A lot of .class attrs is not used, how does it work?
  x. When & Who invoke Runtime.exit
  x. Check again, all native implementations & args to see if the implementation is complete

  x. Problems caused by UTF-8
    java_lang_System::jvm_initProperties Commented out "UTF-8" related content

    xx. Load "sun/nio/cs/ext/ExtendedCharsets" according to the normal process,
        ExtendedCharsets will load a lot of content for a long time, how to optimize?
        Java_lang_Class::forName0 currently skips "sun/nio/cs/ext/ExtendedCharsets"
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
    // println!("main class: {}, args: {:?}", class, args);
    let mut thread = JavaMainThread::new(class.replace(".", util::FILE_SEP), args);
    thread.run();
}

#[cfg(test)]
mod tests {
    use classfile::BytesRef;
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

        let mut ary = Box::new(Vec::new());
        ary.push(100);
        ary.push(99);
        ary.push(98);
        ary.push(97);
        ary.push(96);
        assert_eq!(ary.as_slice(), vec![100, 99, 98, 97, 96].as_slice());
    }

    #[test]
    fn t_arc() {
        use std::sync::{Arc, Mutex};
        struct TestArc {
            bytes: Arc<Vec<u8>>,
        }

        let ref_bytes = {
            let bytes = Arc::new(vec![1, 2, 3, 4]);
            let t = TestArc { bytes };
            let ref_bytes = Some(t.bytes.clone());
            assert_eq!(2, Arc::strong_count(&t.bytes));
            ref_bytes
        };
        assert!(ref_bytes.is_some());
        assert_eq!(ref_bytes, Some(Arc::new(vec![1, 2, 3, 4])));
        assert_eq!(1, Arc::strong_count(&ref_bytes.unwrap()));

        use crate::oop::Oop;
        let null1 = Arc::new(Oop::Null);
        let null2 = Arc::new(Oop::Null);
        assert!(!Arc::ptr_eq(&null1, &null2));
        let null11 = null1.clone();
        assert!(Arc::ptr_eq(&null1, &null11));

        let str1 = Vec::from("hello, world");
        let str1 = Arc::new(str1);
        let v1 = Arc::new(Mutex::new(Box::new(Oop::new_const_utf8(str1))));
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

    #[test]
    fn t_hash() {
        use std::collections::hash_map::DefaultHasher;
        fn calc_hash(s: String) -> u64 {
            let mut hasher = DefaultHasher::new();
            s.hash(&mut hasher);
            hasher.finish()
        }

        let s1 = String::from_utf8_lossy(b"abcde").to_string();
        let s2 = String::from_utf8_lossy(b"abcde").to_string();
        let hash1 = calc_hash(s1);
        let hash2 = calc_hash(s2);
        assert_eq!(hash1, hash2);

        let mut s1 = String::from_utf8_lossy(b"abcde").to_string();
        s1.push_str("1");
        s1.push_str("12");
        s1.push_str("123");
        let mut s2 = String::from_utf8_lossy(b"abcde").to_string();
        s2.push_str("112123");
        let hash1 = calc_hash(s1);
        let hash2 = calc_hash(s2);
        assert_eq!(hash1, hash2);

        let s1 = String::from_utf8_lossy(b"abcde1").to_string();
        let s2 = String::from_utf8_lossy(b"abcde2").to_string();
        let hash1 = calc_hash(s1);
        let hash2 = calc_hash(s2);
        assert_ne!(hash1, hash2)
    }

    #[test]
    fn t_borrow() {
        use std::cell::RefCell;
        use std::sync::{Arc, RwLock};

        struct Frame {
            stack: RefCell<Vec<i32>>,
        }

        let frame = Arc::new(RwLock::new(Box::new(Frame {
            stack: RefCell::new(Vec::new()),
        })));

        let f1 = frame.read().unwrap();
        {
            f1.stack.borrow_mut().push(100);
            f1.stack.borrow_mut().push(200);
            f1.stack.borrow_mut().push(300);
        }

        let _f2 = frame.read().unwrap();
        {
            assert_eq!(f1.stack.borrow().len(), 3);
        }
    }

    #[test]
    fn t_bytes_ref() {
        use std::borrow::{Borrow, Cow};
        use std::collections::HashMap;
        use std::sync::Arc;
        let br1 = Arc::new(Vec::from("abc".as_bytes()));
        let br2 = Arc::new(Vec::from("abc"));
        assert_eq!(br1, br2);

        let mut map: HashMap<BytesRef, &'static str> = HashMap::new();
        map.insert(br1, "abc");
        let item = map.get(&Vec::from("abc"));
        assert!(item.is_some());
        assert_eq!(item, Some(&"abc"));

        //xx1:
        // let k1 = "abc".as_bytes();
        // let k1 = k1.borrow();
        // let item = map.get(&k1);

        // let k1 = "abc";
        // let item = map.get(k1.as_ref());
    }
}
