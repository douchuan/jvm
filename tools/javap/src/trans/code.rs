use super::instruction::{get_instructions, InstructionInfo};
use classfile::{attributes::Code, ClassFile, OpCode};

pub struct Translator<'a> {
    pub cf: &'a ClassFile,
    pub code: &'a Code,
}

impl<'a> Translator<'a> {
    pub fn get(&self) -> Vec<String> {
        let codes = self.code.code.as_slice();
        let infos = self.interp();
        infos.iter().map(|it| it.assemble(codes)).collect()
    }
}

impl<'a> Translator<'a> {
    fn interp(&self) -> Vec<InstructionInfo> {
        let mut infos = vec![];

        let mut instructions = get_instructions();
        let codes = self.code.code.as_slice();
        let codes_len = codes.len();
        let mut pc = 0;
        let mut enable_wide = false;
        loop {
            if pc >= codes_len {
                break;
            }

            let instruction = instructions.get_mut(codes[pc] as usize).unwrap();

            if enable_wide {
                instruction.set_wide(true);
            }

            let (info, new_pc) = instruction.run(codes, pc);

            if enable_wide {
                instruction.set_wide(false);
                enable_wide = false;
            }

            if info.op_code == OpCode::wide {
                enable_wide = true;
            }

            pc = new_pc;
            infos.push(info);
        }

        infos
    }
}
