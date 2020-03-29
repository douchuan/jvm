use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Goto;

impl Instruction for Goto {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::goto,
            icp: 0,
        };

        (info, pc + 3)
    }
}
