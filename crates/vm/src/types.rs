use crate::oop::class::ClassPtr;
use crate::oop::field::FieldId;
use crate::oop::method::MethodId;
use crate::runtime::Frame;
use crate::runtime::JavaThread;
use classfile::ClassFile;
use std::sync::Arc;

pub type FieldIdRef = Arc<FieldId>;
pub type MethodIdRef = Arc<MethodId>;
pub type ClassRef = Arc<ClassPtr>;

def_ref!(ClassFileRef, ClassFile);
def_sync_ref!(FrameRef, Frame);
def_sync_ref!(JavaThreadRef, JavaThread);

// Runtime string allocation
def_ptr!(ByteAry, Vec<u8>);
def_ptr!(BoolAry, Vec<u8>);
def_ptr!(CharAry, Vec<u16>);
def_ptr!(ShortAry, Vec<i16>);
def_ptr!(IntAry, Vec<i32>);
def_ptr!(LongAry, Vec<i64>);
def_ptr!(FloatAry, Vec<f32>);
def_ptr!(DoubleAry, Vec<f64>);
