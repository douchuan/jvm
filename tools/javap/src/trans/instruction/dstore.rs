use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Dstore;

impl Instruction for Dstore {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::dstore.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 2)
    }
}
