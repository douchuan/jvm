use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Ifeq;

impl Instruction for Ifeq {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::ifeq,
            icp: 0,
            wide: false,
        };

        (info, pc + 3)
    }
}
