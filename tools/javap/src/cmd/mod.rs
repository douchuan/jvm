use classfile::ClassFile;

mod disassemble;

pub use disassemble::Disassemble;

pub trait Cmd {
    fn run(&self, cf: ClassFile);
}
