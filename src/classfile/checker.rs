use crate::classfile::types::{CheckResult, ConstantPool};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Err {
    InvalidMagicNum,
    InvalidCpClassNameIdx,
    InvalidCpFieldRefClsIdx,
    InvalidCpFieldRefNameAndTypeIdx,
    InvalidCpMethodRefClsIdx,
    InvalidCpMethodRefNameAndTypeIdx,
    InvalidCpInterfaceMethodRefClsIdx,
    InvalidCpInterfaceMethodRefNameAndTypeIdx,
    InvalidCpStrStrIdx,
    InvalidCpNameAndTypeNameIdx,
    InvalidCpNameAndTypeDescIdx,
    InvalidCpMethodHandleRefKind,
    InvalidCpMethodHandleRefIdx,
    InvalidCpMethodTypeDescIdx,
    InvalidCpInvokeDynBootstrapMethodAttrIdx,
    InvalidCpInvokeDynNameAndTypeIdx,
    InvalidFieldAccFlags,
    InvalidFieldNameIdx,
    InvalidFieldDescIdx,
    InvalidMethodAccFlags,
    InvalidMethodNameIdx,
    InvalidMethodDescIdx,
}

pub trait Checker {
    fn check(&self, cp: &ConstantPool) -> CheckResult;
}
