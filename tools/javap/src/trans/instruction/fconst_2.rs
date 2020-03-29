#![allow(non_camel_case_types)]
use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Fconst_2;

impl Instruction for Fconst_2 {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::fconst_2,
            icp: 0,
        };

        (info, pc + 1)
    }
}
