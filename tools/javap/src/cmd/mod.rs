use classfile::ClassFile;

mod disassemble;
mod line_number;
mod sd;

pub trait Cmd {
    fn run(&self, cf: ClassFile);
}

pub mod factory {
    use super::Cmd;

    use super::disassemble::Disassemble;
    use super::line_number::LineNumber;

    pub fn line_number() -> impl Cmd {
        LineNumber
    }

    pub fn disassemble() -> impl Cmd {
        Disassemble
    }
}
