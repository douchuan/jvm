use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Caload;

impl Instruction for Caload {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::caload,
            icp: 0,
            wide: false,
        };

        (info, pc + 1)
    }
}
