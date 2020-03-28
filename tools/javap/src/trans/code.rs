use classfile::{attributes::Code, ClassFile, OpCode};

struct ByteCode {
    pc: usize,
    name: &'static str,
    icp: Option<usize>,
}

pub struct Translator<'a> {
    cf: &'a ClassFile,
    code: &'a Code,
}

struct Interp {
    pc: usize,
}

impl Interp {
    fn run(&mut self) {
        // let mut instructions = vec![];
    }
}
