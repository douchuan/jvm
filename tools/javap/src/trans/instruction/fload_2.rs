#![allow(non_camel_case_types)]
use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Fload_2;

impl Instruction for Fload_2 {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::fload_2,
            icp: 0,
        };

        (info, pc + 1)
    }
}
