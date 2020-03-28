use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Iinc;

impl Instruction for Iinc {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::iinc.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 3)
    }
}
