#![allow(non_camel_case_types)]
use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Dup2_X1;

impl Instruction for Dup2_X1 {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::dup2_x1,
            icp: 0,
            wide: false,
        };

        (info, pc + 1)
    }
}
