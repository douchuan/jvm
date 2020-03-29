use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Sipush;

impl Instruction for Sipush {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::sipush,
            icp: 0,
        };

        (info, pc + 3)
    }
}
