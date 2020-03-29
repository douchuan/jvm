#![allow(non_camel_case_types)]
use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct If_Acmpeq;

impl Instruction for If_Acmpeq {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::if_acmpeq,
            icp: 0,
        };

        (info, pc + 3)
    }
}
