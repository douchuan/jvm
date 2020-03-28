use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Goto_W;

impl Instruction for Goto_W {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::goto_w.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 5)
    }
}
