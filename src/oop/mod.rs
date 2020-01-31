#![allow(unused)]

use std::sync::{Arc, Condvar, Mutex};

use crate::classfile::{types::*, ClassFile};
use crate::runtime::ClassLoader;

pub mod class;
pub mod consts;
pub mod field;
pub mod method;

pub use self::class::{Class, ClassKind};
pub use self::field::FieldIdRef;
pub use self::method::MethodIdRef;

macro_rules! def_ref {
    ($name:ident, $t:ty) => {
        pub type $name = Arc<Mutex<Box<$t>>>;
    };
}

#[macro_export]
macro_rules! new_ref {
    ($name:ident) => {
        std::sync::Arc::new(std::sync::Mutex::new(Box::new($name)));
    };
}

pub type ClassFileRef = Arc<ClassFile>;

def_ref!(ClassRef, Class);
def_ref!(OopRef, OopDesc);

#[derive(Debug, Clone, Copy)]
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

    pub fn new_mirror(cls_obj: ClassRef, n: usize) -> OopRef {
        let mut filed_values = Vec::with_capacity(n);
        for _ in 0..n {
            filed_values.push(consts::get_null());
        }

        let v = MirrorOopDesc {
            class: cls_obj,
            filed_values
        };
        Self::new(Oop::Mirror(v))
    }

    pub fn new_null() -> OopRef {
        Self::new(Oop::Null)
    }

    fn new(v: Oop) -> OopRef {
        Arc::new(Mutex::new(Box::new(Self {
            v,
            cond: Condvar::new(),
            monitor: Mutex::new(0),
        })))
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
            //todo: 'j' allowed??
            b'J' | b'j' => ValueType::LONG,
            b'F' => ValueType::FLOAT,
            b'D' => ValueType::DOUBLE,
            b'V' => ValueType::VOID,
            b'L' => ValueType::OBJECT,
            b'[' => ValueType::ARRAY,
            t => {
                /*
                let mut v = Vec::new();
                v.push(*t);
                trace!("ValueType = {}", String::from_utf8_lossy(v.as_slice()));
                */
                unreachable!()
            },
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
    class: ClassRef,
    filed_values: Vec<OopRef>,
}

impl InstOopDesc {
    pub fn new(class: ClassRef) -> Self {
        let n = {
            let class = class.lock().unwrap();
            match &class.kind {
                class::ClassKind::Instance(class_obj) => class_obj.n_inst_fields,
                _ => unreachable!(),
            }
        };

        let mut field_values = Vec::with_capacity(n);
        for _ in 0..n {
            field_values.push(consts::get_null());
        }

        Self {
            class,
            field_values
        }
    }
}

impl ArrayOopDesc {
    pub fn new(class: ClassRef, elements: Vec<OopRef>) -> Self {
        {
            assert!(class.lock().unwrap().is_array());
        }

        Self {
            class,
            elements,
        }
    }

    pub fn get_dimension(&self) -> usize {
        let class = self.class.lock().unwrap();
        match &class.kind {
            class::ClassKind::ObjectArray(ary_class_obj) => ary_class_obj.get_dimension().unwrap(),
            class::ClassKind::TypeArray(ary_class_obj) => ary_class_obj.get_dimension().unwrap(),
            _ => unreachable!()
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

pub fn init() {
    consts::init();
}
