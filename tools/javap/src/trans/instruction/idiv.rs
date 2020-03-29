use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Idiv;

impl Instruction for Idiv {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::idiv,
            icp: 0,
        };

        (info, pc + 1)
    }
}
