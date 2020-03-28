use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Ior;

impl Instruction for Ior {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::ior.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}
