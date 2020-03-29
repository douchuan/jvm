#![allow(non_camel_case_types)]
use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Fstore_0;

impl Instruction for Fstore_0 {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::fstore_0,
            icp: 0,
        };

        (info, pc + 1)
    }
}
