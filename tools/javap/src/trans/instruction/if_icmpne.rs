#![allow(non_camel_case_types)]
use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct If_Icmpne;

impl Instruction for If_Icmpne {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::if_icmpne,
            icp: 0,
        };

        (info, pc + 3)
    }
}
