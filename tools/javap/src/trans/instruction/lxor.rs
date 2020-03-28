use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Lxor;

impl Instruction for Lxor {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::lxor.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}
