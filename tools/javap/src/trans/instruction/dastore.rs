use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Dastore;

impl Instruction for Dastore {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::dastore,
            icp: 0,
        };

        (info, pc + 1)
    }
}
