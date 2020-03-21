use classfile::flags as class_flags;
use classfile::ClassFile;

type AccFlag = u16;

pub struct Translator<'a> {
    cf: &'a ClassFile,
}

impl<'a> Translator<'a> {
    pub fn new(cf: &'a ClassFile) -> Self {
        Self { cf }
    }
}

pub trait AccFlagHelper {
    fn is_public(&self) -> bool;
    fn is_final(&self) -> bool;
    fn is_super(&self) -> bool;
    fn is_interface(&self) -> bool;
    fn is_abstract(&self) -> bool;
    fn is_synthetic(&self) -> bool;
    fn is_enum(&self) -> bool;
}

impl<'a> Translator<'a> {
    pub fn get(&self) -> String {
        let flags = self.cf.acc_flags;

        let mut name = String::new();

        if flags.is_public() {
            name.push_str("public");
        }

        if flags.is_final() {
            name.push_str(" final");
        }

        if flags.is_interface() {
            name.push_str(" interface");
        } else if flags.is_enum() {
            // name.push_str(" enum");
            unimplemented!()
        } else {
            if flags.is_abstract() {
                name.push_str(" abstract class");
            } else {
                name.push_str(" class")
            }
        }

        name
    }
}

impl AccFlagHelper for AccFlag {
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
}
