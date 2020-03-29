use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Laload;

impl Instruction for Laload {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::laload,
            icp: 0,
            wide: false,
        };

        (info, pc + 1)
    }
}
