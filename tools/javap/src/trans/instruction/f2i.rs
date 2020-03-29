use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct F2I;

impl Instruction for F2I {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::f2i,
            icp: 0,
        };

        (info, pc + 1)
    }
}
