use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Dmul;

impl Instruction for Dmul {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::dmul,
            icp: 0,
            wide: false,
        };

        (info, pc + 1)
    }
}
