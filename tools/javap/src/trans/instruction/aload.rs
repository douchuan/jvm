use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Aload;

impl Instruction for Aload {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::aload.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 2)
    }
}
