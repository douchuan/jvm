use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Daload;

impl Instruction for Daload {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::daload.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 2)
    }
}
