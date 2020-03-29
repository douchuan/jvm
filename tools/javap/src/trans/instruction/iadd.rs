use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Iadd;

impl Instruction for Iadd {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::iadd,
            icp: 0,
            wide: false,
        };

        (info, pc + 1)
    }
}
