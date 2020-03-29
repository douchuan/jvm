use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct D2F;

impl Instruction for D2F {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::d2f,
            icp: 0,
        };

        (info, pc + 1)
    }
}
