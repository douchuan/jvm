use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Ifeq;

impl Instruction for Ifeq {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::ifeq.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 3)
    }
}
