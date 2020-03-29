use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Pop;

impl Instruction for Pop {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::pop,
            icp: 0,
            wide: false,
        };

        (info, pc + 1)
    }
}
