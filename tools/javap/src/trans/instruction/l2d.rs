use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct L2D;

impl Instruction for L2D {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::l2d.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}
