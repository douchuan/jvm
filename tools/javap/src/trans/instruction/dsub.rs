use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Dsub;

impl Instruction for Dsub {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::dsub,
            icp: 0,
        };

        (info, pc + 1)
    }
}
