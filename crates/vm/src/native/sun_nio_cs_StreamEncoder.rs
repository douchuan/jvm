#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::Oop;
use crate::runtime::{self, require_class3};

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![new_fn(
        "forOutputStreamWriter",
        "(Ljava/io/OutputStream;Ljava/lang/Object;Ljava/lang/String;)Lsun/nio/cs/StreamEncoder;",
        Box::new(jvm_forOutputStreamWriter),
    )]
}

fn jvm_forOutputStreamWriter(_env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    let os = args.get(0).unwrap();
    let obj = args.get(1).unwrap();
    let _charset_name = args.get(2).unwrap();

    let charset_cls = require_class3(None, b"java/nio/charset/Charset").unwrap();
    let default_charset_oop = {
        let cls = charset_cls.read().unwrap();
        let id = cls.get_field_id(b"defaultCharset", b"Ljava/nio/charset/Charset;", true);
        cls.get_static_field_value(id)
    };

    //check defaultCharset
    match default_charset_oop {
        Oop::Ref(_) => (),
        _ => unreachable!(),
    }

    let encoder = require_class3(None, b"sun/nio/cs/StreamEncoder").unwrap();
    let encoder_oop = Oop::new_inst(encoder.clone());
    let args = vec![
        encoder_oop.clone(),
        os.clone(),
        obj.clone(),
        default_charset_oop,
    ];

    runtime::invoke::invoke_ctor(
        encoder,
        b"(Ljava/io/OutputStream;Ljava/lang/Object;Ljava/nio/charset/Charset;)V",
        args,
    );

    Ok(Some(encoder_oop))
}
