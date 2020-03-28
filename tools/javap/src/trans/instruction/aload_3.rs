use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Aload_3;

impl Instruction for Aload_3 {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::aload_3.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}
