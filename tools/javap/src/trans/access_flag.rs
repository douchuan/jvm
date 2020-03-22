use classfile::flags as class_flags;

pub type AccessFlag = u16;

pub trait AccessFlagHelper {
    fn is_public(&self) -> bool;
    fn is_final(&self) -> bool;
    fn is_super(&self) -> bool;
    fn is_interface(&self) -> bool;
    fn is_abstract(&self) -> bool;
    fn is_synthetic(&self) -> bool;
    fn is_enum(&self) -> bool;
    fn is_native(&self) -> bool;
}

impl AccessFlagHelper for AccessFlag {
    fn is_public(&self) -> bool {
        (*self & class_flags::ACC_PUBLIC) != 0
    }

    fn is_final(&self) -> bool {
        (*self & class_flags::ACC_FINAL) != 0
    }

    fn is_super(&self) -> bool {
        (*self & class_flags::ACC_SUPER) != 0
    }

    fn is_interface(&self) -> bool {
        (*self & class_flags::ACC_INTERFACE) != 0
    }

    fn is_abstract(&self) -> bool {
        (*self & class_flags::ACC_ABSTRACT) != 0
    }

    fn is_synthetic(&self) -> bool {
        (*self & class_flags::ACC_SYNTHETIC) != 0
    }

    fn is_enum(&self) -> bool {
        (*self & class_flags::ACC_ENUM) != 0
    }

    fn is_native(&self) -> bool {
        (*self & class_flags::ACC_NATIVE) != 0
    }
}