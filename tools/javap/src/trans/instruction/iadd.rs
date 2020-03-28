use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Iadd;

impl Instruction for Iadd {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::iadd.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}
