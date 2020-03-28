#![allow(non_camel_case_types)]
use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Return_Void;

impl Instruction for Return_Void {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::return_void.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}
