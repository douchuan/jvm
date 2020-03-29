use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Aastore;

impl Instruction for Aastore {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::aastore,
            icp: 0,
        };

        (info, pc + 1)
    }
}
