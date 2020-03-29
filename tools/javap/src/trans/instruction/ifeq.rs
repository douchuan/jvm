use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Ifeq;

impl Instruction for Ifeq {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::ifeq,
            icp: 0,
        };

        (info, pc + 3)
    }
}
