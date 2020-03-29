#![allow(non_camel_case_types)]
use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Fstore_1;

impl Instruction for Fstore_1 {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::fstore_1,
            icp: 0,
        };

        (info, pc + 1)
    }
}
