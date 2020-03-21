use classfile::ClassFile;

use crate::cmd::Cmd;
use crate::trans;

pub struct LineNumber;

impl Cmd for LineNumber {
    fn run(&self, cf: ClassFile) {
        let source_file = trans::class_source_file(&cf);
        let this_class = trans::class_this_class(&cf);
        let super_class = trans::class_super_class(&cf);
        // let access_flags = trans::access_flags(&cf);
        // trace!("source file = {}", source_file);

        trace!("this class  = {}", this_class);
        trace!("super class  = {}", super_class);
        // trace!("class access flags = {}", access_flags);
    }
}
