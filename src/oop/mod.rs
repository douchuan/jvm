#![allow(unused)]

use std::sync::{Arc, Condvar, Mutex};

use crate::classfile::ClassFile;
use crate::runtime::{require_class3, ClassLoader};
use crate::types::*;

pub mod class;
pub mod consts;
pub mod field;
pub mod method;

pub use self::class::{Class, ClassKind};

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
pub enum ValueType {
    BYTE,
    BOOLEAN,
    CHAR,
    SHORT,
    INT,
    LONG,
    FLOAT,
    DOUBLE,
    VOID,
    OBJECT,
    ARRAY,
}

#[derive(Debug)]
pub enum RefDesc {
    Inst(InstOopDesc),
    Array(ArrayOopDesc),
    TypeArray(TypeArrayValue),
    Mirror(MirrorOopDesc),
}

#[derive(Debug)]
pub struct OopRefDesc {
    pub v: RefDesc,
    pub hash_code: Option<i32>,

    //这两个所相关的field，有意义吗？本身操作，就隐含了lock
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

    Ref(Arc<Mutex<OopRefDesc>>),
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
        Self::new_ref(RefDesc::Inst(v))
    }

    pub fn new_ref_ary(ary_cls_obj: ClassRef, len: usize) -> Oop {
        let mut elements = Vec::with_capacity(len);
        for _ in 0..len {
            elements.push(consts::get_null());
        }

        Self::new_ref_ary2(ary_cls_obj, elements)
    }

    pub fn new_ref_ary2(ary_cls_obj: ClassRef, elms: Vec<Oop>) -> Oop {
        let v = ArrayOopDesc::new(ary_cls_obj, elms);
        Self::new_ref(RefDesc::Array(v))
    }

    pub fn new_mirror(target: ClassRef) -> Oop {
        let java_lang_class = require_class3(None, b"java/lang/Class").unwrap();
        let field_values = field::build_inited_field_values(java_lang_class);
        let v = MirrorOopDesc {
            target: Some(target),
            field_values,
            value_type: ValueType::OBJECT,
        };

        Self::new_ref(RefDesc::Mirror(v))
    }

    pub fn new_prim_mirror(value_type: ValueType) -> Oop {
        let java_lang_class = require_class3(None, b"java/lang/Class").unwrap();
        let field_values = field::build_inited_field_values(java_lang_class);
        let v = MirrorOopDesc {
            target: None,
            field_values: vec![],
            value_type,
        };

        Self::new_ref(RefDesc::Mirror(v))
    }

    pub fn new_ary_mirror(target: ClassRef, value_type: ValueType) -> Oop {
        let java_lang_class = require_class3(None, b"java/lang/Class").unwrap();
        let field_values = field::build_inited_field_values(java_lang_class);
        let v = MirrorOopDesc {
            target: Some(target),
            field_values: vec![],
            value_type: value_type,
        };

        Self::new_ref(RefDesc::Mirror(v))
    }

    pub fn char_ary_from1(v: &[u16]) -> Oop {
        let elms = Vec::from(v);
        Self::new_char_ary2(elms)
    }

    pub fn new_byte_ary(len: usize) -> Oop {
        let mut elms = Vec::with_capacity(len);
        for _ in 0..len {
            elms.push(0);
        }
        Self::new_byte_ary2(elms)
    }

    pub fn new_bool_ary(len: usize) -> Oop {
        let mut elms = Vec::with_capacity(len);
        for _ in 0..len {
            elms.push(0);
        }
        Self::new_bool_ary2(elms)
    }

    pub fn new_char_ary(len: usize) -> Oop {
        let mut elms = Vec::with_capacity(len);
        for _ in 0..len {
            elms.push(0);
        }
        Self::new_char_ary2(elms)
    }

    pub fn new_short_ary(len: usize) -> Oop {
        let mut elms = Vec::with_capacity(len);
        for _ in 0..len {
            elms.push(0);
        }
        Self::new_short_ary2(elms)
    }

    pub fn new_int_ary(len: usize) -> Oop {
        let mut elms = Vec::with_capacity(len);
        for _ in 0..len {
            elms.push(0);
        }
        Self::new_int_ary2(elms)
    }

    pub fn new_float_ary(len: usize) -> Oop {
        let mut elms = Vec::with_capacity(len);
        for _ in 0..len {
            elms.push(0.0);
        }
        Self::new_float_ary2(elms)
    }

    pub fn new_double_ary(len: usize) -> Oop {
        let mut elms = Vec::with_capacity(len);
        for _ in 0..len {
            elms.push(0.0);
        }
        Self::new_double_ary2(elms)
    }

    pub fn new_long_ary(len: usize) -> Oop {
        let mut elms = Vec::with_capacity(len);
        for _ in 0..len {
            elms.push(0);
        }
        Self::new_long_ary2(elms)
    }

    pub fn new_byte_ary2(elms: Vec<u8>) -> Oop {
        let ary = Box::new(elms);
        let v = TypeArrayValue::Byte(ary);
        Self::new_ref(RefDesc::TypeArray(v))
    }

    pub fn new_bool_ary2(elms: Vec<u8>) -> Oop {
        let ary = Box::new(elms);
        let v = TypeArrayValue::Bool(ary);
        Self::new_ref(RefDesc::TypeArray(v))
    }

    pub fn new_char_ary2(elms: Vec<u16>) -> Oop {
        let ary = Box::new(elms);
        let v = TypeArrayValue::Char(ary);
        Self::new_ref(RefDesc::TypeArray(v))
    }

    pub fn new_short_ary2(elms: Vec<i16>) -> Oop {
        let ary = Box::new(elms);
        let v = TypeArrayValue::Short(ary);
        Self::new_ref(RefDesc::TypeArray(v))
    }

    pub fn new_int_ary2(elms: Vec<i32>) -> Oop {
        let ary = Box::new(elms);
        let v = TypeArrayValue::Int(ary);
        Self::new_ref(RefDesc::TypeArray(v))
    }

    pub fn new_float_ary2(elms: Vec<f32>) -> Oop {
        let ary = Box::new(elms);
        let v = TypeArrayValue::Float(ary);
        Self::new_ref(RefDesc::TypeArray(v))
    }

    pub fn new_double_ary2(elms: Vec<f64>) -> Oop {
        let ary = Box::new(elms);
        let v = TypeArrayValue::Double(ary);
        Self::new_ref(RefDesc::TypeArray(v))
    }

    pub fn new_long_ary2(elms: Vec<i64>) -> Oop {
        let ary = Box::new(elms);
        let v = TypeArrayValue::Long(ary);
        Self::new_ref(RefDesc::TypeArray(v))
    }

    fn new_ref(v: RefDesc) -> Oop {
        let v = OopRefDesc {
            v,
            cond: Condvar::new(),
            monitor: Mutex::new(0),
            hash_code: None,
        };

        let v = Arc::new(Mutex::new(v));
        Oop::Ref(v)
    }
}

impl OopRefDesc {
    pub fn monitor_enter(&mut self) {
        let mut v = self.monitor.lock().unwrap();
        *v += 1;
    }

    pub fn monitor_exit(&mut self) {
        let mut v = self.monitor.lock().unwrap();
        *v -= 1;
    }
}

impl From<&u8> for ValueType {
    fn from(v: &u8) -> Self {
        match v {
            b'B' => ValueType::BYTE,
            b'Z' => ValueType::BOOLEAN,
            b'C' => ValueType::CHAR,
            b'S' => ValueType::SHORT,
            b'I' => ValueType::INT,
            b'J' => ValueType::LONG,
            b'F' => ValueType::FLOAT,
            b'D' => ValueType::DOUBLE,
            b'V' => ValueType::VOID,
            b'L' => ValueType::OBJECT,
            b'[' => ValueType::ARRAY,
            t => {
                let s = [*t];
                let s = String::from_utf8_lossy(&s);
                unreachable!("Unknown ValueType = {}", s)
            }
        }
    }
}

impl Into<&[u8]> for ValueType {
    fn into(self) -> &'static [u8] {
        match self {
            ValueType::BYTE => b"B",
            ValueType::BOOLEAN => b"Z",
            ValueType::CHAR => b"C",
            ValueType::SHORT => b"S",
            ValueType::INT => b"I",
            ValueType::LONG => b"J",
            ValueType::FLOAT => b"F",
            ValueType::DOUBLE => b"D",
            ValueType::VOID => b"V",
            ValueType::OBJECT => b"L",
            ValueType::ARRAY => b"[",
        }
    }
}

impl ValueType {
    pub fn parse_wrap(class_loader: Option<ClassLoader>, desc: &str) -> Self {
        match desc.as_bytes().first().unwrap() {
            b'B' | b'Z' | b'C' | b'S' | b'I' => ValueType::INT,
            b'J' => ValueType::LONG,
            b'F' => ValueType::FLOAT,
            b'D' => ValueType::DOUBLE,
            b'V' => ValueType::VOID,
            b'L' => ValueType::OBJECT,
            b'[' => ValueType::ARRAY,
            _ => unreachable!(),
        }
    }

    /*
    pub fn into_primitive_name(&self) -> &'static [u8] {
        match *self {
            ValueType::BYTE => b"byte",
            ValueType::BOOLEAN => b"boolean",
            ValueType::CHAR => b"char",
            ValueType::SHORT => b"short",
            ValueType::INT => b"int",
            ValueType::LONG => b"long",
            ValueType::FLOAT => b"float",
            ValueType::DOUBLE => b"double",
            ValueType::VOID => b"void",
            ValueType::OBJECT => unreachable!(),
            ValueType::ARRAY => unreachable!(),
        }
    }
    */
}

#[derive(Debug, Clone)]
pub struct InstOopDesc {
    pub class: ClassRef,
    pub field_values: Vec<Oop>,
}

#[derive(Debug, Clone)]
pub struct ArrayOopDesc {
    pub class: ClassRef,
    pub elements: Vec<Oop>,
}

#[derive(Debug, Clone)]
pub enum TypeArrayValue {
    Byte(ByteAry),
    Bool(BoolAry),
    Char(CharAry),
    Short(ShortAry),
    Float(FloatAry),
    Double(DoubleAry),
    Int(IntAry),
    Long(LongAry),
}

#[derive(Debug, Clone)]
pub struct MirrorOopDesc {
    pub target: Option<ClassRef>,
    pub field_values: Vec<Oop>,
    pub value_type: ValueType,
}

impl InstOopDesc {
    pub fn new(class: ClassRef) -> Self {
        let field_values = field::build_inited_field_values(class.clone());

        Self {
            class,
            field_values,
        }
    }
}

impl ArrayOopDesc {
    pub fn new(class: ClassRef, elements: Vec<Oop>) -> Self {
        {
            assert!(class.lock().unwrap().is_array());
        }

        Self { class, elements }
    }

    pub fn get_dimension(&self) -> usize {
        let class = self.class.lock().unwrap();
        match &class.kind {
            class::ClassKind::ObjectArray(ary_class_obj) => ary_class_obj.get_dimension(),
            class::ClassKind::TypeArray(ary_class_obj) => ary_class_obj.get_dimension(),
            _ => unreachable!(),
        }
    }
}

impl MirrorOopDesc {
    pub fn is_prim_mirror(&self) -> bool {
        self.target.is_none()
    }
}

impl TypeArrayValue {
    pub fn len(&self) -> usize {
        match self {
            TypeArrayValue::Char(ary) => ary.len(),
            TypeArrayValue::Byte(ary) => ary.len(),
            TypeArrayValue::Bool(ary) => ary.len(),
            TypeArrayValue::Short(ary) => ary.len(),
            TypeArrayValue::Float(ary) => ary.len(),
            TypeArrayValue::Double(ary) => ary.len(),
            TypeArrayValue::Int(ary) => ary.len(),
            TypeArrayValue::Long(ary) => ary.len(),
        }
    }
}

pub fn init() {
    consts::init();
}
