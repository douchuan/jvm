#![allow(non_camel_case_types)]
use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct If_Icmpeq;

impl Instruction for If_Icmpeq {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::if_icmpeq,
            icp: 0,
        };

        (info, pc + 3)
    }
}
