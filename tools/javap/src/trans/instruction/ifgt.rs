use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Ifgt;

impl Instruction for Ifgt {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::ifgt,
            icp: 0,
        };

        (info, pc + 3)
    }
}
