use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Irem;

impl Instruction for Irem {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::irem,
            icp: 0,
            wide: false,
        };

        (info, pc + 1)
    }
}
