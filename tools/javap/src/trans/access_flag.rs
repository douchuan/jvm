use classfile::flags as class_flags;

pub struct Translator {
    flags: AccessFlag,
}

impl Translator {
    pub fn new(flags: AccessFlag) -> Self {
        Self { flags }
    }
}

impl Translator {
    pub fn class_access_flags(&self) -> String {
        let flags = self.flags;

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
            //todo: impl me
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

    pub fn method_access_flags(&self) -> String {
        let flags = self.flags;

        let mut name = String::new();

        if flags.is_public() {
            name.push_str("public");
        } else if flags.is_protected() {
            name.push_str("protected");
        } else if flags.is_private() {
            name.push_str("private");
        }

        if flags.is_final() {
            name.push_str(" final");
        } else if flags.is_abstract() {
            name.push_str(" abstract");
        }

        name
    }
}

type AccessFlag = u16;

pub trait AccessFlagHelper {
    fn is_public(&self) -> bool;
    fn is_final(&self) -> bool;
    fn is_super(&self) -> bool;
    fn is_interface(&self) -> bool;
    fn is_abstract(&self) -> bool;
    fn is_synthetic(&self) -> bool;
    fn is_annotation(&self) -> bool;
    fn is_enum(&self) -> bool;
    fn is_private(&self) -> bool;
    fn is_protected(&self) -> bool;
    fn is_static(&self) -> bool;
    fn is_synchronized(&self) -> bool;
    fn is_bridge(&self) -> bool;
    fn is_varargs(&self) -> bool;
    fn is_native(&self) -> bool;
    fn is_strict(&self) -> bool;
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

    fn is_annotation(&self) -> bool {
        (*self & class_flags::ACC_ANNOTATION) != 0
    }

    fn is_enum(&self) -> bool {
        (*self & class_flags::ACC_ENUM) != 0
    }

    fn is_private(&self) -> bool {
        (*self & class_flags::ACC_PRIVATE) != 0
    }

    fn is_protected(&self) -> bool {
        (*self & class_flags::ACC_PROTECTED) != 0
    }

    fn is_static(&self) -> bool {
        (*self & class_flags::ACC_STATIC) != 0
    }

    fn is_synchronized(&self) -> bool {
        (*self & class_flags::ACC_SYNCHRONIZED) != 0
    }

    fn is_bridge(&self) -> bool {
        (*self & class_flags::ACC_BRIDGE) != 0
    }

    fn is_varargs(&self) -> bool {
        (*self & class_flags::ACC_VARARGS) != 0
    }

    fn is_native(&self) -> bool {
        (*self & class_flags::ACC_NATIVE) != 0
    }

    fn is_strict(&self) -> bool {
        (*self & class_flags::ACC_STRICT) != 0
    }
}
