#![allow(non_camel_case_types)]
use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Dconst_1;

impl Instruction for Dconst_1 {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::dconst_1,
            icp: 0,
        };

        (info, pc + 1)
    }
}
