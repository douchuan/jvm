use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Pop;

impl Instruction for Pop {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::pop.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}
