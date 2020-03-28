use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Fload_0;

impl Instruction for Fload_0 {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::fload_0.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}
