use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Irem;

impl Instruction for Irem {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::irem,
            icp: 0,
        };

        (info, pc + 1)
    }
}
