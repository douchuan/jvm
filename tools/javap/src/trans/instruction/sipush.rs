use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Sipush;

impl Instruction for Sipush {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::sipush,
            icp: 0,
            wide: false,
        };

        (info, pc + 3)
    }
}
