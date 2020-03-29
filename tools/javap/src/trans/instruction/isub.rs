use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Isub;

impl Instruction for Isub {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::isub,
            icp: 0,
        };

        (info, pc + 1)
    }
}
