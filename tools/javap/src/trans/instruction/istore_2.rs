#![allow(non_camel_case_types)]
use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Istore_2;

impl Instruction for Istore_2 {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::istore_2,
            icp: 0,
            wide: false,
        };

        (info, pc + 1)
    }
}
