use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Lmul;

impl Instruction for Lmul {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::lmul,
            icp: 0,
            wide: false,
        };

        (info, pc + 1)
    }
}
