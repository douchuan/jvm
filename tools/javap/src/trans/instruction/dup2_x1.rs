use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Dup2_X1;

impl Instruction for Dup2_X1 {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::dup2_x1.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}
