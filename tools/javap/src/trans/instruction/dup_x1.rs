#![allow(non_camel_case_types)]
use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Dup_X1;

impl Instruction for Dup_X1 {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::dup_x1,
            icp: 0,
        };

        (info, pc + 1)
    }
}
