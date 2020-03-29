use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Dcmpg;

impl Instruction for Dcmpg {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::dcmpg,
            icp: 0,
        };

        (info, pc + 1)
    }
}
