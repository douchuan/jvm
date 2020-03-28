#![allow(non_camel_case_types)]
use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Dup_X2;

impl Instruction for Dup_X2 {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::dup_x2.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}
