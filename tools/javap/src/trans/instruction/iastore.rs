use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Iastore;

impl Instruction for Iastore {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::iastore,
            icp: 0,
        };

        (info, pc + 1)
    }
}
