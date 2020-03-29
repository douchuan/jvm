use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Lsub;

impl Instruction for Lsub {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::lsub,
            icp: 0,
        };

        (info, pc + 1)
    }
}
