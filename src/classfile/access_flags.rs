use crate::classfile::types::*;

macro_rules! def_acc {
    ($name:ident, $v:expr) => {
        pub const $name: U2 = $v;
    };
}

def_acc!(ACC_PUBLIC, 0x0001);
def_acc!(ACC_PRIVATE, 0x0002);
def_acc!(ACC_PROTECTED, 0x0004);
def_acc!(ACC_STATIC, 0x0008);
def_acc!(ACC_FINAL, 0x0010);
def_acc!(ACC_SYNCHRONIZED, 0x0020);
def_acc!(ACC_SUPER, 0x0020);
def_acc!(ACC_VOLATILE, 0x0040);
def_acc!(ACC_BRIDGE, 0x0040);
def_acc!(ACC_VARARGS, 0x0080);
def_acc!(ACC_TRANSIENT, 0x0080);
def_acc!(ACC_NATIVE, 0x0100);
def_acc!(ACC_INTERFACE, 0x0200);
def_acc!(ACC_ABSTRACT, 0x0400);
def_acc!(ACC_STRICT, 0x0800);
def_acc!(ACC_SYNTHETIC, 0x1000);
def_acc!(ACC_ANNOTATION, 0x2000);
def_acc!(ACC_ENUM, 0x4000);
def_acc!(ACC_MIRANDA, 0x8000);
def_acc!(ACC_REFLECT_MASK, 0xffff);
