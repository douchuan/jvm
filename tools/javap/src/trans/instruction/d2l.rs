use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct D2L;

impl Instruction for D2L {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::d2l,
            icp: 0,
            wide: false,
        };

        (info, pc + 1)
    }
}
