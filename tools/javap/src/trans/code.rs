use super::instruction::{get_instructions, InstructionInfo};
use classfile::{attributes::Code, ClassFile};

pub struct Translator<'a> {
    pub cf: &'a ClassFile,
    pub code: &'a Code,
}

impl<'a> Translator<'a> {
    pub fn get(&self) -> Vec<InstructionInfo> {
        self.interp()
    }
}

impl<'a> Translator<'a> {
    fn interp(&self) -> Vec<InstructionInfo> {
        let mut infos = vec![];

        let instructions = get_instructions();
        let codes = self.code.code.as_slice();
        let codes_len = codes.len();
        let mut pc = 0;
        loop {
            if pc >= codes_len {
                break;
            }

            let instruction = instructions.get(codes[pc] as usize).unwrap();
            let (info, new_pc) = instruction.run(codes, pc);
            pc = new_pc;
            infos.push(info);
        }

        infos
    }
}
