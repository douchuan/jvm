use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Iconst_4;

impl Instruction for Iconst_4 {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::iconst_4.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}
