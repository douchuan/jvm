use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Pop2;

impl Instruction for Pop2 {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::pop2,
            icp: 0,
        };

        (info, pc + 1)
    }
}
