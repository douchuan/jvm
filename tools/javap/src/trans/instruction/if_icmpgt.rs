#![allow(non_camel_case_types)]
use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct If_Icmpgt;

impl Instruction for If_Icmpgt {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::if_icmpgt,
            icp: 0,
        };

        (info, pc + 3)
    }
}
