#![allow(non_camel_case_types)]
use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Jsr_W;

impl Instruction for Jsr_W {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::jsr_w,
            icp: 0,
        };

        (info, pc + 5)
    }
}
