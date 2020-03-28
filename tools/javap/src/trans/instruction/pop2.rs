use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Pop2;

impl Instruction for Pop2 {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::pop2.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}
