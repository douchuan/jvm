use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Fload;

impl Instruction for Fload {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::fload.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 2)
    }
}
