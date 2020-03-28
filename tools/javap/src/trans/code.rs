use classfile::{attributes::Code, ClassFile};

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
