#![allow(non_camel_case_types)]
use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Fstore_1;

impl Instruction for Fstore_1 {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::fstore_1.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}
