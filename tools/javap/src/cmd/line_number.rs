use classfile::ClassFile;

use crate::cmd::Cmd;

pub struct LineNumber;

impl Cmd for LineNumber {
    fn run(&self, cf: ClassFile) {
        unimplemented!()
    }
}
