use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Dneg;

impl Instruction for Dneg {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::dneg.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}
