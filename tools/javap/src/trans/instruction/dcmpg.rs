use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Dcmpg;

impl Instruction for Dcmpg {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::dcmpg.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}
