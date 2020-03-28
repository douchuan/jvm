use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Fdiv;

impl Instruction for Fdiv {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::fdiv.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}
