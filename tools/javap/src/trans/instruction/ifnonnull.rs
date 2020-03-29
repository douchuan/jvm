use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Ifnonnull;

impl Instruction for Ifnonnull {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::ifnonnull,
            icp: 0,
        };

        (info, pc + 3)
    }
}
