use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Swap;

impl Instruction for Swap {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::swap,
            icp: 0,
        };

        (info, pc + 1)
    }
}
