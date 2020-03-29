use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Ior;

impl Instruction for Ior {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::ior,
            icp: 0,
            wide: false,
        };

        (info, pc + 1)
    }
}
