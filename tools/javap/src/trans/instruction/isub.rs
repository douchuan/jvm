use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Isub;

impl Instruction for Isub {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::isub.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}
