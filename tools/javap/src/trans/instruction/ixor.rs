use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Ixor;

impl Instruction for Ixor {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::ixor.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}
