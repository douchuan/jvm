#![allow(non_camel_case_types)]
use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Lstore_2;

impl Instruction for Lstore_2 {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::lstore_2,
            icp: 0,
        };

        (info, pc + 1)
    }
}
