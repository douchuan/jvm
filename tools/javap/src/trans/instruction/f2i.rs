use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct F2I;

impl Instruction for F2I {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::f2i.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}
