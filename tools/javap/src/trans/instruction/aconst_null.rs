#![allow(non_camel_case_types)]
use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Aconst_Null;

impl Instruction for Aconst_Null {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::aconst_null.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}
