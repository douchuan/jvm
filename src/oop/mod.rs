#![allow(unused)]

use std::sync::{Arc, Condvar, Mutex, RwLock};

use crate::classfile::ClassFile;
use crate::runtime::{require_class3, ClassLoader};
use crate::types::*;

pub mod ary;
pub mod class;
pub mod consts;
pub mod field;
pub mod inst;
pub mod method;
pub mod mirror;
pub mod values;

pub use self::ary::{ArrayOopDesc, TypeArrayDesc};
pub use self::class::{Class, ClassKind};
pub use self::inst::InstOopDesc;
pub use self::mirror::MirrorOopDesc;
pub use self::values::ValueType;

#[derive(Debug)]
pub enum RefKind {
    Inst(InstOopDesc),
    Array(ArrayOopDesc),
    TypeArray(TypeArrayDesc),
    Mirror(MirrorOopDesc),
}

#[derive(Debug)]
pub struct RefKindDesc {
    pub v: RefKind,
    pub hash_code: Option<i32>,

    // Do these two fields make sense? The operation itself, implicit lock
    cond: Condvar,
    monitor: Mutex<usize>,
}

#[derive(Debug, Clone)]
pub enum Oop {
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    /*
        used by: Throwable.java
    private static final String NULL_CAUSE_MESSAGE = "Cannot suppress a null exception.";
        */
    ConstUtf8(BytesRef),
    //used by oop::field::Filed::get_constant_value
    Null,

    Ref(OopRef),
}

impl Oop {
    pub fn new_int(v: i32) -> Self {
        Oop::Int(v)
    }

    pub fn new_long(v: i64) -> Self {
        Oop::Long(v)
    }

    pub fn new_float(v: f32) -> Self {
        Oop::Float(v)
    }

    pub fn new_double(v: f64) -> Self {
        Oop::Double(v)
    }

    pub fn new_const_utf8(v: BytesRef) -> Self {
        Oop::ConstUtf8(v)
    }

    pub fn new_null() -> Self {
        Oop::Null
    }

    pub fn new_inst(cls_obj: ClassRef) -> Oop {
        let v = InstOopDesc::new(cls_obj);
        Self::new_ref(RefKind::Inst(v))
    }

    pub fn new_ref_ary(ary_cls_obj: ClassRef, len: usize) -> Oop {
        let mut elements = Vec::with_capacity(len);
        for _ in 0..len {
            elements.push(consts::get_null());
        }
        // elements.resize(0, consts::get_null());
        Self::new_ref_ary2(ary_cls_obj, elements)
    }

    pub fn new_ref_ary2(ary_cls_obj: ClassRef, elms: Vec<Oop>) -> Oop {
        let v = ArrayOopDesc::new(ary_cls_obj, elms);
        Self::new_ref(RefKind::Array(v))
    }

    pub fn new_mirror(target: ClassRef) -> Oop {
        let java_lang_class = require_class3(None, b"java/lang/Class").unwrap();
        let field_values = field::build_inited_field_values(java_lang_class);
        let v = MirrorOopDesc {
            target: Some(target),
            field_values,
            value_type: ValueType::OBJECT,
        };

        Self::new_ref(RefKind::Mirror(v))
    }

    pub fn new_prim_mirror(value_type: ValueType, target: Option<ClassRef>) -> Oop {
        let java_lang_class = require_class3(None, b"java/lang/Class").unwrap();
        let field_values = field::build_inited_field_values(java_lang_class);
        let v = MirrorOopDesc {
            target,
            field_values,
            value_type,
        };

        Self::new_ref(RefKind::Mirror(v))
    }

    pub fn new_ary_mirror(target: ClassRef, value_type: ValueType) -> Oop {
        let java_lang_class = require_class3(None, b"java/lang/Class").unwrap();
        let field_values = field::build_inited_field_values(java_lang_class);
        let v = MirrorOopDesc {
            target: Some(target),
            field_values: vec![],
            value_type: value_type,
        };

        Self::new_ref(RefKind::Mirror(v))
    }

    pub fn char_ary_from1(v: &[u16]) -> Oop {
        let elms = Vec::from(v);
        Self::new_char_ary2(elms)
    }

    pub fn new_byte_ary(len: usize) -> Oop {
        let mut elms = Vec::with_capacity(len);
        elms.resize(len, 0);
        Self::new_byte_ary2(elms)
    }

    pub fn new_bool_ary(len: usize) -> Oop {
        let mut elms = Vec::with_capacity(len);
        elms.resize(len, 0);
        Self::new_bool_ary2(elms)
    }

    pub fn new_char_ary(len: usize) -> Oop {
        let mut elms = Vec::with_capacity(len);
        elms.resize(len, 0);
        Self::new_char_ary2(elms)
    }

    pub fn new_short_ary(len: usize) -> Oop {
        let mut elms = Vec::with_capacity(len);
        elms.resize(len, 0);
        Self::new_short_ary2(elms)
    }

    pub fn new_int_ary(len: usize) -> Oop {
        let mut elms = Vec::with_capacity(len);
        elms.resize(len, 0);
        Self::new_int_ary2(elms)
    }

    pub fn new_float_ary(len: usize) -> Oop {
        let mut elms = Vec::with_capacity(len);
        elms.resize(len, 0.0);
        Self::new_float_ary2(elms)
    }

    pub fn new_double_ary(len: usize) -> Oop {
        let mut elms = Vec::with_capacity(len);
        elms.resize(len, 0.0);
        Self::new_double_ary2(elms)
    }

    pub fn new_long_ary(len: usize) -> Oop {
        let mut elms = Vec::with_capacity(len);
        elms.resize(len, 0);
        Self::new_long_ary2(elms)
    }

    pub fn new_byte_ary2(elms: Vec<u8>) -> Oop {
        let ary = Box::new(elms);
        let v = TypeArrayDesc::Byte(ary);
        Self::new_ref(RefKind::TypeArray(v))
    }

    pub fn new_bool_ary2(elms: Vec<u8>) -> Oop {
        let ary = Box::new(elms);
        let v = TypeArrayDesc::Bool(ary);
        Self::new_ref(RefKind::TypeArray(v))
    }

    pub fn new_char_ary2(elms: Vec<u16>) -> Oop {
        let ary = Box::new(elms);
        let v = TypeArrayDesc::Char(ary);
        Self::new_ref(RefKind::TypeArray(v))
    }

    pub fn new_short_ary2(elms: Vec<i16>) -> Oop {
        let ary = Box::new(elms);
        let v = TypeArrayDesc::Short(ary);
        Self::new_ref(RefKind::TypeArray(v))
    }

    pub fn new_int_ary2(elms: Vec<i32>) -> Oop {
        let ary = Box::new(elms);
        let v = TypeArrayDesc::Int(ary);
        Self::new_ref(RefKind::TypeArray(v))
    }

    pub fn new_float_ary2(elms: Vec<f32>) -> Oop {
        let ary = Box::new(elms);
        let v = TypeArrayDesc::Float(ary);
        Self::new_ref(RefKind::TypeArray(v))
    }

    pub fn new_double_ary2(elms: Vec<f64>) -> Oop {
        let ary = Box::new(elms);
        let v = TypeArrayDesc::Double(ary);
        Self::new_ref(RefKind::TypeArray(v))
    }

    pub fn new_long_ary2(elms: Vec<i64>) -> Oop {
        let ary = Box::new(elms);
        let v = TypeArrayDesc::Long(ary);
        Self::new_ref(RefKind::TypeArray(v))
    }

    fn new_ref(v: RefKind) -> Oop {
        let v = RefKindDesc {
            v,
            cond: Condvar::new(),
            monitor: Mutex::new(0),
            hash_code: None,
        };

        let v = Arc::new(RwLock::new(Box::new(v)));
        Oop::Ref(v)
    }
}

impl RefKindDesc {
    pub fn monitor_enter(&mut self) {
        let mut v = self.monitor.lock().unwrap();
        *v += 1;
    }

    pub fn monitor_exit(&mut self) {
        let mut v = self.monitor.lock().unwrap();
        *v -= 1;
    }
}

pub fn init() {
    consts::init();
}
