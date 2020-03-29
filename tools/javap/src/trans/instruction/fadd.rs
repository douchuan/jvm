use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Fadd;

impl Instruction for Fadd {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::fadd,
            icp: 0,
            wide: false,
        };

        (info, pc + 1)
    }
}
