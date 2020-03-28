use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Aload_2;

impl Instruction for Aload_2 {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::aload_2.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}
