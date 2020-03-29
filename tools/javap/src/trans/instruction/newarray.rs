use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Newarray;

impl Instruction for Newarray {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::newarray,
            icp: 0,
        };

        (info, pc + 2)
    }
}
