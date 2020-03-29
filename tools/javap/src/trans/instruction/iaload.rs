use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Iaload;

impl Instruction for Iaload {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::iaload,
            icp: 0,
            wide: false,
        };

        (info, pc + 1)
    }
}
