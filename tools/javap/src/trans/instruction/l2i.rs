use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct L2I;

impl Instruction for L2I {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::l2i.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}
