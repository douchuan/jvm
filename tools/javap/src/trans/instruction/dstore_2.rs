use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Dstore_2;

impl Instruction for Dstore_2 {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::dstore_2.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}
