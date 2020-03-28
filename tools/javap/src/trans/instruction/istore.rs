use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Istore;

impl Instruction for Istore {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::istore.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 2)
    }
}
