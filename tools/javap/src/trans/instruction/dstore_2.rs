#![allow(non_camel_case_types)]
use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Dstore_2;

impl Instruction for Dstore_2 {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::dstore_2,
            icp: 0,
        };

        (info, pc + 1)
    }
}
