use crate::misc::SysInfo;
use classfile::ClassFile;

mod disassemble;

pub use disassemble::Disassemble;

pub trait Cmd {
    fn run(&self, si: &SysInfo, cf: ClassFile);
}
