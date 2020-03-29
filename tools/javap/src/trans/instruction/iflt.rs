use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Iflt;

impl Instruction for Iflt {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::iflt,
            icp: 0,
            wide: false,
        };

        (info, pc + 3)
    }
}
