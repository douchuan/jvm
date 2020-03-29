use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Ifge;

impl Instruction for Ifge {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::ifge,
            icp: 0,
            wide: false,
        };

        (info, pc + 3)
    }
}
