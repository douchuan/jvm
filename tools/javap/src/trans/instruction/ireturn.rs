use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Ireturn;

impl Instruction for Ireturn {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::ireturn,
            icp: 0,
        };

        (info, pc + 1)
    }
}
