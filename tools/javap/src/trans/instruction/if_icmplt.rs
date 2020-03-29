#![allow(non_camel_case_types)]
use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct If_Icmplt;

impl Instruction for If_Icmplt {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::if_icmplt,
            icp: 0,
            wide: false,
        };

        (info, pc + 3)
    }
}
