use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Ladd;

impl Instruction for Ladd {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::ladd,
            icp: 0,
        };

        (info, pc + 1)
    }
}
