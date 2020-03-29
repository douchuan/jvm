use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Athrow;

impl Instruction for Athrow {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::athrow,
            icp: 0,
        };

        (info, pc + 1)
    }
}
