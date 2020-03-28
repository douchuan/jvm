use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Astore;

impl Instruction for Astore {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::astore.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 2)
    }
}
