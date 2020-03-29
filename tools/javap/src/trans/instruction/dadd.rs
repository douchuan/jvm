use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Dadd;

impl Instruction for Dadd {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::dadd,
            icp: 0,
        };

        (info, pc + 1)
    }
}
