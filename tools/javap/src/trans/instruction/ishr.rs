use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Ishr;

impl Instruction for Ishr {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::ishr.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}
