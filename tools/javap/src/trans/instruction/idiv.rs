use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Idiv;

impl Instruction for Idiv {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::idiv.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}
