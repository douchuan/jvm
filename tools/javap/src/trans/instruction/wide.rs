use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Wide;

impl Instruction for Wide {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::wide,
            icp: 0,
        };

        (info, pc + 1)
    }
}
