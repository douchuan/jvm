#![allow(non_camel_case_types)]
use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Istore_3;

impl Instruction for Istore_3 {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::istore_3.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}
