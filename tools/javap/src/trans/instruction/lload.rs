use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Lload;

impl Instruction for Lload {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::lload.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 2)
    }
}
