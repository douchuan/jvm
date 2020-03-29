use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Nop;

impl Instruction for Nop {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::nop,
            icp: 0,
        };

        (info, pc + 1)
    }
}
