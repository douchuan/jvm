#![allow(non_camel_case_types)]
use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Iconst_M1;

impl Instruction for Iconst_M1 {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::iconst_m1,
            icp: 0,
            wide: false,
        };

        (info, pc + 1)
    }
}
