use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Ishl;

impl Instruction for Ishl {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::ishl.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}