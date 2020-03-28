#![allow(non_camel_case_types)]
use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct If_Icmpeq;

impl Instruction for If_Icmpeq {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::if_icmpeq.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 3)
    }
}
