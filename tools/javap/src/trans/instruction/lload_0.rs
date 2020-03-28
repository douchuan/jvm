#![allow(non_camel_case_types)]
use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Lload_0;

impl Instruction for Lload_0 {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::lload_0.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}
