use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Lor;

impl Instruction for Lor {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::lor.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}
