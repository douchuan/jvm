use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Caload;

impl Instruction for Caload {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::caload,
            icp: 0,
        };

        (info, pc + 1)
    }
}
