#![allow(non_camel_case_types)]
use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Lstore_0;

impl Instruction for Lstore_0 {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::lstore_0,
            icp: 0,
        };

        (info, pc + 1)
    }
}
