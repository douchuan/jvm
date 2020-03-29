use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Ineg;

impl Instruction for Ineg {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::ineg,
            icp: 0,
        };

        (info, pc + 1)
    }
}
