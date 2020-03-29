use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Fastore;

impl Instruction for Fastore {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::fastore,
            icp: 0,
        };

        (info, pc + 1)
    }
}
