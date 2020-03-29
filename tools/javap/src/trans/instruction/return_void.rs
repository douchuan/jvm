#![allow(non_camel_case_types)]
use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Return_Void;

impl Instruction for Return_Void {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::return_void,
            icp: 0,
        };

        (info, pc + 1)
    }
}
