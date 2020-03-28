use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Dsub;

impl Instruction for Dsub {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::dsub.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}
