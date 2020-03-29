#![allow(non_camel_case_types)]
use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Goto_W;

impl Instruction for Goto_W {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::goto_w,
            icp: 0,
        };

        (info, pc + 5)
    }
}
