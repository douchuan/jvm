use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Iastore;

impl Instruction for Iastore {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::iastore.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}
