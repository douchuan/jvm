use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Fcmpg;

impl Instruction for Fcmpg {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::fcmpg,
            icp: 0,
        };

        (info, pc + 1)
    }
}
