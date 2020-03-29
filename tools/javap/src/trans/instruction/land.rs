use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Land;

impl Instruction for Land {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::land,
            icp: 0,
        };

        (info, pc + 1)
    }
}
