use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Astore_2;

impl Instruction for Astore_2 {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::astore_2.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}
