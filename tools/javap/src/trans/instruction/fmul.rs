use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Fmul;

impl Instruction for Fmul {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::fmul.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}
