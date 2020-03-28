use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Lmul;

impl Instruction for Lmul {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::lmul.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}