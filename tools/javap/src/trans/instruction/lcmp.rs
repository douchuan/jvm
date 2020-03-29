use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Lcmp;

impl Instruction for Lcmp {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::lcmp,
            icp: 0,
        };

        (info, pc + 1)
    }
}
