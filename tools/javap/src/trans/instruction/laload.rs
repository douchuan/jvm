use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Laload;

impl Instruction for Laload {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::laload,
            icp: 0,
        };

        (info, pc + 1)
    }
}
