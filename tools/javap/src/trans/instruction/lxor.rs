use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Lxor;

impl Instruction for Lxor {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::lxor,
            icp: 0,
            wide: false,
        };

        (info, pc + 1)
    }
}
