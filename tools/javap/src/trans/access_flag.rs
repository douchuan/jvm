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
        let mut parts = vec![];
        let flags = self.flags;

        if flags.is_public() {
            parts.push("public");
        }

        if flags.is_final() {
            parts.push("final");
        }

        if flags.is_interface() {
            parts.push("interface");
        } else if flags.is_enum() {
            parts.push("class");
        } else {
            if flags.is_abstract() {
                parts.push("abstract class");
            } else {
                parts.push("class")
            }
        }

        parts.join(" ")
    }

    pub fn method_access_flags(&self) -> String {
        let mut parts = vec![];
        let flags = self.flags;

        if flags.is_public() {
            parts.push("public");
        } else if flags.is_protected() {
            parts.push("protected");
        } else if flags.is_private() {
            parts.push("private");
        }

        if flags.is_static() {
            parts.push("static");
        }

        if flags.is_native() {
            parts.push("native");
        }

        if flags.is_final() {
            parts.push("final");
        } else if flags.is_abstract() {
            parts.push("abstract");
        }

        parts.join(" ")
    }

    pub fn field_access_flags(&self) -> String {
        let mut parts = vec![];
        let flags = self.flags;

        if flags.is_public() {
            parts.push("public");
        } else if flags.is_protected() {
            parts.push("protected");
        } else if flags.is_private() {
            parts.push("private");
        }

        if flags.is_static() {
            parts.push("static");
        }

        if flags.is_final() {
            parts.push("final");
        }

        parts.join(" ")
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
    fn is_package(&self) -> bool;
    fn compare(&self, other: u16) -> i32;
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

    fn is_package(&self) -> bool {
        *self == 0
    }

    //compare access permission
    //  *self >  other,  1
    //  *self == other,  0
    //  *self <  other, -1
    fn compare(&self, other: u16) -> i32 {
        let flags = *self;
        if flags == other {
            return 0;
        }

        if flags == 0 {
            if other.is_private() {
                return 1;
            }
        } else if flags.is_public() {
            if !other.is_public() {
                return 1;
            }
        } else if flags.is_protected() {
            if other == 0 || other.is_private() {
                return 1;
            }
        }

        -1
    }
}
