use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Iaload;

impl Instruction for Iaload {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::iaload.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}
