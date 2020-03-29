use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Iand;

impl Instruction for Iand {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::iand,
            icp: 0,
        };

        (info, pc + 1)
    }
}
