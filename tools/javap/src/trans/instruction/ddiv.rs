use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Ddiv;

impl Instruction for Ddiv {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::ddiv,
            icp: 0,
        };

        (info, pc + 1)
    }
}
