#![allow(unused)]

use std::sync::{Arc, Mutex};

use crate::classfile::{types::*, ClassFile};
use crate::runtime::class_loader;

pub mod class;
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
    Str(String),
    TypeArray(ArrayOopDesc),
    ObjectArray(ArrayOopDesc),
    Null,
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
    pub fn parse_wrap(class_loader: Option<class_loader::ClassLoader>, desc: &str) -> Self {
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
pub struct ArrayOopDesc {
    class: ClassRef,
    elements: Vec<Oop>,
}

impl ArrayOopDesc {

    pub fn new(class: ClassRef, len: usize) -> Self {
        Self {
            class,
            elements: Vec::with_capacity(len)
        }
    }

    pub fn get_dimension(&self) -> usize {
        self.class.lock().unwrap().get_dimension().unwrap()
    }

    pub fn get_length(&self) -> usize {
        self.elements.len()
    }

    pub fn get_elm_at(&self, index: usize) -> Option<&Oop> {
        self.elements.get(index)
    }

    pub fn set_elm_at(&mut self, index: usize, elm: Oop) {
        self.elements[index] = elm;
    }
}

