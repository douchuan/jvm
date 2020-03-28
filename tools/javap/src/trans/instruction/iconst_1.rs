use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Iconst_1;

impl Instruction for Iconst_1 {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::iconst_1.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}
