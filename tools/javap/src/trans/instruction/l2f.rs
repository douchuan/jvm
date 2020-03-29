use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct L2F;

impl Instruction for L2F {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::l2f,
            icp: 0,
        };

        (info, pc + 1)
    }
}
