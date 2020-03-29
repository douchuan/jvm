use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Ifnonnull;

impl Instruction for Ifnonnull {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::ifnonnull,
            icp: 0,
            wide: false,
        };

        (info, pc + 3)
    }
}
