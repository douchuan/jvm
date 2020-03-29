use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Drem;

impl Instruction for Drem {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::drem,
            icp: 0,
        };

        (info, pc + 1)
    }
}
