use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Faload;

impl Instruction for Faload {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::faload,
            icp: 0,
            wide: false,
        };

        (info, pc + 1)
    }
}
