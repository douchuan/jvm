#![allow(non_camel_case_types)]
use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct If_Icmpne;

impl Instruction for If_Icmpne {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::if_icmpne,
            icp: 0,
            wide: false,
        };

        (info, pc + 3)
    }
}
