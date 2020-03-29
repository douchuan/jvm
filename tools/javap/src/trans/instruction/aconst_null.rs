#![allow(non_camel_case_types)]
use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Aconst_Null;

impl Instruction for Aconst_Null {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::aconst_null,
            icp: 0,
        };

        (info, pc + 1)
    }
}
