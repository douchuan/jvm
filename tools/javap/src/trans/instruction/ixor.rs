use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Ixor;

impl Instruction for Ixor {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::ixor,
            icp: 0,
            wide: false,
        };

        (info, pc + 1)
    }
}
