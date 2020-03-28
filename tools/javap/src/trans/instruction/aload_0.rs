use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Aload_0;

impl Instruction for Aload_0 {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::aload_0.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}
