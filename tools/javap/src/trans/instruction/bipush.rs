use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Bipush;

impl Instruction for Bipush {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::bipush,
            icp: 0,
        };

        (info, pc + 2)
    }
}
