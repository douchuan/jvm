use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Lor;

impl Instruction for Lor {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::lor,
            icp: 0,
            wide: false,
        };

        (info, pc + 1)
    }
}
