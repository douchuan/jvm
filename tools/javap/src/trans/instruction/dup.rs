use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Dup;

impl Instruction for Dup {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::dup,
            icp: 0,
            wide: false,
        };

        (info, pc + 1)
    }
}
