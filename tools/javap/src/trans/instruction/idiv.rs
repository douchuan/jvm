use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Idiv;

impl Instruction for Idiv {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::idiv,
            icp: 0,
        };

        (info, pc + 1)
    }
}
