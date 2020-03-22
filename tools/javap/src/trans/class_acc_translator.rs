use classfile::flags as class_flags;
use classfile::ClassFile;
use crate::trans::AccessFlagHelper;

pub struct Translator<'a> {
    cf: &'a ClassFile,
}

impl<'a> Translator<'a> {
    pub fn new(cf: &'a ClassFile) -> Self {
        Self { cf }
    }
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


