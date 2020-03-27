use crate::cmd::Cmd;
use crate::trans::{self, AccessFlagHelper};
use classfile::ClassFile;

pub struct Disassemble;

impl Cmd for Disassemble {
    fn run(&self, cf: ClassFile) {
        if cf.acc_flags.is_interface() {
            self.render_interface(cf);
        } else if cf.acc_flags.is_enum() {
            self.render_enum(cf);
        } else {
            self.render_class(cf);
        }
    }
}

impl Disassemble {
    fn render_interface(&self, cf: ClassFile) {
        unimplemented!()
    }

    fn render_enum(&self, cf: ClassFile) {
        unimplemented!()
    }

    fn render_class(&self, cf: ClassFile) {
        unimplemented!()
    }
}
