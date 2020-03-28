use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Lconst_0;

impl Instruction for Lconst_0 {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::lconst_0.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}
