use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Iflt;

impl Instruction for Iflt {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::iflt.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 3)
    }
}
