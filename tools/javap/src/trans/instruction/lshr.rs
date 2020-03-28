use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Lshr;

impl Instruction for Lshr {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::lshr.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}