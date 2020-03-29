use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Lastore;

impl Instruction for Lastore {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::lastore,
            icp: 0,
        };

        (info, pc + 1)
    }
}
