#![allow(non_camel_case_types)]
use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Astore_0;

impl Instruction for Astore_0 {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::astore_0,
            icp: 0,
        };

        (info, pc + 1)
    }
}
