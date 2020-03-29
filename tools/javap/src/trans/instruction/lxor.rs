use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Lxor;

impl Instruction for Lxor {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::lxor,
            icp: 0,
        };

        (info, pc + 1)
    }
}
