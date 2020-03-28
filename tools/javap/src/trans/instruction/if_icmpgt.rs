#![allow(non_camel_case_types)]
use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct If_Icmpgt;

impl Instruction for If_Icmpgt {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::if_icmpgt.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 3)
    }
}
