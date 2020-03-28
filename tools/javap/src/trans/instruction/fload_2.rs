#![allow(non_camel_case_types)]
use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Fload_2;

impl Instruction for Fload_2 {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::fload_2.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}
