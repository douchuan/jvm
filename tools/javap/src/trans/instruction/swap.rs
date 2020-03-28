use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Swap;

impl Instruction for Swap {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::swap.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}
