use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Iload;

impl Instruction for Iload {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::iload.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 2)
    }
}
