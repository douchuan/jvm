use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Lrem;

impl Instruction for Lrem {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::lrem.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}
