use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Lneg;

impl Instruction for Lneg {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::lneg,
            icp: 0,
            wide: false,
        };

        (info, pc + 1)
    }
}
