use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Ixor;

impl Instruction for Ixor {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::ixor,
            icp: 0,
        };

        (info, pc + 1)
    }
}
