use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Imul;

impl Instruction for Imul {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::imul,
            icp: 0,
        };

        (info, pc + 1)
    }
}
