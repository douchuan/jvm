use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct F2L;

impl Instruction for F2L {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::f2l.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}
