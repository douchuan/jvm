use crate::oop::class::Class;
use crate::oop::field::FieldId;
use crate::oop::method::MethodId;
use crate::oop::RefKindDesc;
use crate::runtime::DataArea;
use class_parser::ClassFile;
use std::cell::RefCell;
use std::sync::Arc;

pub type FieldIdRef = Arc<FieldId>;
pub type MethodIdRef = Arc<MethodId>;
pub type DataAreaRef = RefCell<DataArea>;

def_ref!(ClassFileRef, ClassFile);
def_sync_ref!(ClassRef, Class);
def_sync_ref!(OopRef, RefKindDesc);

// Runtime string allocation
def_ptr!(ByteAry, Vec<u8>);
def_ptr!(BoolAry, Vec<u8>);
def_ptr!(CharAry, Vec<u16>);
def_ptr!(ShortAry, Vec<i16>);
def_ptr!(IntAry, Vec<i32>);
def_ptr!(LongAry, Vec<i64>);
def_ptr!(FloatAry, Vec<f32>);
def_ptr!(DoubleAry, Vec<f64>);
