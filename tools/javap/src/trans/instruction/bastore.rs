use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Bastore;

impl Instruction for Bastore {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::bastore,
            icp: 0,
        };

        (info, pc + 1)
    }
}
