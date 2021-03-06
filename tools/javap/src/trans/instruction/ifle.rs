use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Ifle;

impl Instruction for Ifle {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::ifle,
            icp: 0,
            wide: false,
        };

        (info, pc + 3)
    }
}
