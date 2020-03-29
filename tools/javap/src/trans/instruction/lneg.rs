use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Lneg;

impl Instruction for Lneg {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::lneg,
            icp: 0,
        };

        (info, pc + 1)
    }
}
