use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Ireturn;

impl Instruction for Ireturn {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::ireturn,
            icp: 0,
        };

        (info, pc + 1)
    }
}
