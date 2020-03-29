use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Saload;

impl Instruction for Saload {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::saload,
            icp: 0,
        };

        (info, pc + 1)
    }
}
