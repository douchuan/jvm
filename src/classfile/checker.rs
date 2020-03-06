use crate::types::ConstantPool;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Err {
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

pub type CheckResult = Result<(), Err>;

pub trait Checker {
    fn check(&self, cp: &ConstantPool) -> CheckResult;
}
