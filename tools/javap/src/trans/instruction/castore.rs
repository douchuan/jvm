use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Castore;

impl Instruction for Castore {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::castore,
            icp: 0,
        };

        (info, pc + 1)
    }
}
