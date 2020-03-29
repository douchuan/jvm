use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Nop;

impl Instruction for Nop {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::nop,
            icp: 0,
            wide: false,
        };

        (info, pc + 1)
    }
}
