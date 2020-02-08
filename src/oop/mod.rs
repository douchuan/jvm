#![allow(unused)]

use std::sync::{Arc, Condvar, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::classfile::{types::*, ClassFile};
use crate::runtime::{ClassLoader, require_class3};

pub mod class;
pub mod consts;
pub mod field;
pub mod method;

pub use self::class::{Class, ClassKind};
pub use self::field::FieldIdRef;
pub use self::method::MethodIdRef;

def_ref!(ClassFileRef, ClassFile);
def_sync_ref!(ClassRef, Class);
def_sync_ref!(OopRef, OopDesc);

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

#[derive(Debug, Clone)]
pub enum Oop {
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Str(BytesRef),
    Inst(InstOopDesc),

    //todo: optimise me, create a TypeArray
    Array(ArrayOopDesc),

    Mirror(MirrorOopDesc),

    //used by oop::field::Filed::get_constant_value
    Null,
}

#[derive(Debug)]
pub struct OopDesc {
    pub v: Oop,
    cond: Condvar,
    monitor: Mutex<usize>,

    pub hash_code: i32,
}

impl OopDesc {
    pub fn new_int(v: i32) -> OopRef {
        Self::new(Oop::Int(v))
    }

    pub fn new_long(v: i64) -> OopRef {
        Self::new(Oop::Long(v))
    }

    pub fn new_float(v: f32) -> OopRef {
        Self::new(Oop::Float(v))
    }

    pub fn new_double(v: f64) -> OopRef {
        Self::new(Oop::Double(v))
    }

    pub fn new_str(v: BytesRef) -> OopRef {
        Self::new(Oop::Str(v))
    }

    pub fn new_inst(cls_obj: ClassRef) -> OopRef {
        let v = InstOopDesc::new(cls_obj);
        Self::new(Oop::Inst(v))
    }

    pub fn new_ary(ary_cls_obj: ClassRef, len: usize) -> OopRef {
        let mut elements = Vec::with_capacity(len);
        //todo: optimize me
        for _ in 0..len {
            elements.push(consts::get_null());
        }

        Self::new_ary2(ary_cls_obj, elements)
    }

    pub fn new_ary2(ary_cls_obj: ClassRef, elms: Vec<OopRef>) -> OopRef {
        let v = ArrayOopDesc::new(ary_cls_obj, elms);
        Self::new(Oop::Array(v))
    }

    pub fn new_mirror(target: ClassRef) -> OopRef {
        let java_lang_class = require_class3(None, b"java/lang/Class").unwrap();
        let field_values = field::build_inited_field_values(java_lang_class);
        let v = MirrorOopDesc {
            target: Some(target),
            field_values,
            value_type: ValueType::OBJECT,
        };

        Self::new(Oop::Mirror(v))
    }

    pub fn new_prim_mirror(value_type: ValueType) -> OopRef {
        let java_lang_class = require_class3(None, b"java/lang/Class").unwrap();
        let field_values = field::build_inited_field_values(java_lang_class);
        let v = MirrorOopDesc {
            target: None,
            field_values: vec![],
            value_type,
        };

        Self::new(Oop::Mirror(v))
    }

    pub fn new_ary_mirror(target: ClassRef, value_type: ValueType) -> OopRef {
        let java_lang_class = require_class3(None, b"java/lang/Class").unwrap();
        let field_values = field::build_inited_field_values(java_lang_class);
        let v = MirrorOopDesc {
            target: Some(target),
            field_values: vec![],
            value_type: value_type,
        };

        Self::new(Oop::Mirror(v))
    }

    pub fn new_null() -> OopRef {
        Self::new(Oop::Null)
    }

    fn new(v: Oop) -> OopRef {
        //todo: how calc hashcode ?
        let start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let hash_code = start.as_secs() as i32;
        let v = Self {
            v,
            cond: Condvar::new(),
            monitor: Mutex::new(0),
            hash_code,
        };
        new_sync_ref!(v)
    }
}

impl OopDesc {
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
}

#[derive(Debug, Clone)]
pub struct InstOopDesc {
    pub class: ClassRef,
    field_values: Vec<OopRef>,
}

#[derive(Debug, Clone)]
pub struct ArrayOopDesc {
    pub class: ClassRef,
    pub elements: Vec<OopRef>,
}

#[derive(Debug, Clone)]
pub struct MirrorOopDesc {
    pub target: Option<ClassRef>,
    field_values: Vec<OopRef>,
    value_type: ValueType,
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
    pub fn new(class: ClassRef, elements: Vec<OopRef>) -> Self {
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

    pub fn get_length(&self) -> usize {
        self.elements.len()
    }

    pub fn get_elm_at(&self, index: usize) -> OopRef {
        self.elements[index].clone()
    }

    pub fn set_elm_at(&mut self, index: usize, elm: OopRef) {
        self.elements[index] = elm;
    }
}

impl MirrorOopDesc {
    pub fn is_prim_mirror(&self) -> bool {
        self.target.is_none()
    }
}


pub fn init() {
    consts::init();
}
