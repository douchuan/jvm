#![allow(non_camel_case_types)]
use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct If_Icmpge;

impl Instruction for If_Icmpge {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::if_icmpge,
            icp: 0,
        };

        (info, pc + 3)
    }
}
