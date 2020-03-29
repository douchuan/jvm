use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Aaload;

impl Instruction for Aaload {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::aaload,
            icp: 0,
        };

        (info, pc + 1)
    }
}
