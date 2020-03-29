use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Lrem;

impl Instruction for Lrem {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::lrem,
            icp: 0,
        };

        (info, pc + 1)
    }
}
