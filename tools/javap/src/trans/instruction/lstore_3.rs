#![allow(non_camel_case_types)]
use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Lstore_3;

impl Instruction for Lstore_3 {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::lstore_3,
            icp: 0,
        };

        (info, pc + 1)
    }
}
