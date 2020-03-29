use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Fcmpg;

impl Instruction for Fcmpg {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::fcmpg,
            icp: 0,
        };

        (info, pc + 1)
    }
}
