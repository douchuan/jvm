#![allow(unused)]

use std::sync::{Arc, Condvar, Mutex};

use crate::classfile::{types::*, ClassFile};
use crate::runtime::ClassLoader;

pub mod class;
pub mod consts;
pub mod field;
pub mod method;

pub use self::class::ClassObject;
pub use self::field::{Field, FieldId};
pub use self::method::{Method, MethodId};

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

def_ref!(ClassRef, ClassObject);

#[derive(Debug, Clone)]
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
    pub fn new_int(v: i32) -> Arc<Self> {
        Self::new(Oop::Int(v))
    }

    pub fn new_long(v: i64) -> Arc<Self> {
        Self::new(Oop::Long(v))
    }

    pub fn new_float(v: f32) -> Arc<Self> {
        Self::new(Oop::Float(v))
    }

    pub fn new_double(v: f64) -> Arc<Self> {
        Self::new(Oop::Double(v))
    }

    pub fn new_str(v: BytesRef) -> Arc<Self> {
        Self::new(Oop::Str(v))
    }

    pub fn new_inst(v: InstOopDesc) -> Arc<Self> {
        Self::new(Oop::Inst(v))
    }

    pub fn new_ary(v: ArrayOopDesc) -> Arc<Self> {
        Self::new(Oop::Array(v))
    }

    pub fn new_null() -> Arc<Self> {
        Self::new(Oop::Null)
    }

    fn new(v: Oop) -> Arc<Self> {
        Arc::new(Self {
            v,
            cond: Condvar::new(),
            monitor: Mutex::new(0),
        })
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
            _ => unreachable!(),
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
pub struct InstOopDesc {}

#[derive(Debug, Clone)]
pub struct ArrayOopDesc {
    class: ClassRef,
    elements: Vec<Arc<OopDesc>>,
}

impl ArrayOopDesc {
    pub fn new(class: ClassRef, len: usize) -> Self {
        Self {
            class,
            elements: Vec::with_capacity(len),
        }
    }

    pub fn get_dimension(&self) -> usize {
        self.class.lock().unwrap().get_dimension().unwrap()
    }

    pub fn get_length(&self) -> usize {
        self.elements.len()
    }

    pub fn get_elm_at(&self, index: usize) -> Arc<OopDesc> {
        self.elements[index].clone()
    }

    pub fn set_elm_at(&mut self, index: usize, elm: Arc<OopDesc>) {
        self.elements[index] = elm;
    }
}

pub fn init() {
    consts::init();
}
