use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Ldc;

impl Instruction for Ldc {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::ldc,
            icp: codes[pc + 1] as usize,
        };

        (info, pc + 2)
    }
}
