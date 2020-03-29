#![allow(non_camel_case_types)]
use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Iconst_1;

impl Instruction for Iconst_1 {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::iconst_1,
            icp: 0,
            wide: false,
        };

        (info, pc + 1)
    }
}
