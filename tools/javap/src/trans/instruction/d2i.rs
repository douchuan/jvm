use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct D2I;

impl Instruction for D2I {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::d2i,
            icp: 0,
        };

        (info, pc + 1)
    }
}
