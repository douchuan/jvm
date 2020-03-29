use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Fmul;

impl Instruction for Fmul {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::fmul,
            icp: 0,
        };

        (info, pc + 1)
    }
}
