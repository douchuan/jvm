use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct F2L;

impl Instruction for F2L {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::f2l,
            icp: 0,
        };

        (info, pc + 1)
    }
}
