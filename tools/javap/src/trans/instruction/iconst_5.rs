#![allow(non_camel_case_types)]
use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Iconst_5;

impl Instruction for Iconst_5 {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::iconst_5,
            icp: 0,
            wide: false,
        };

        (info, pc + 1)
    }
}
