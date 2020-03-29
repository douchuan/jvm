use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Baload;

impl Instruction for Baload {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::baload,
            icp: 0,
            wide: false,
        };

        (info, pc + 1)
    }
}
