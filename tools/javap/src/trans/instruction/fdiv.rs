use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Fdiv;

impl Instruction for Fdiv {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::fdiv,
            icp: 0,
        };

        (info, pc + 1)
    }
}
