use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct L2D;

impl Instruction for L2D {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::l2d,
            icp: 0,
            wide: false,
        };

        (info, pc + 1)
    }
}
