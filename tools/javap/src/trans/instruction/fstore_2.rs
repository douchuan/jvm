#![allow(non_camel_case_types)]
use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Fstore_2;

impl Instruction for Fstore_2 {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::fstore_2,
            icp: 0,
            wide: false,
        };

        (info, pc + 1)
    }
}
