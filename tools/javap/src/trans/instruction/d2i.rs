use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct D2I;

impl Instruction for D2I {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::d2i,
            icp: 0,
            wide: false,
        };

        (info, pc + 1)
    }
}
