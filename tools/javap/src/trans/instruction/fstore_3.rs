use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Fstore_3;

impl Instruction for Fstore_3 {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::fstore_3.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}
