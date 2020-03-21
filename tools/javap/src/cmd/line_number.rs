use classfile::ClassFile;

use crate::cmd::Cmd;
use crate::trans;

pub struct LineNumber;

impl Cmd for LineNumber {
    fn run(&self, cf: ClassFile) {
        let source_file = trans::extract_source_file(&cf);
        let this_class = trans::extract_this_class_name(&cf);
        // trace!("source file = {}", source_file);
        trace!("this class  = {}", this_class);
    }
}
