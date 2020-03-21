use classfile::ClassFile;

mod line_number;

pub trait Cmd {
    fn run(&self, cf: ClassFile);
}

pub mod factory {
    use super::Cmd;

    use crate::cmd::line_number::LineNumber;

    pub fn line_number() -> impl Cmd {
        LineNumber
    }
}
