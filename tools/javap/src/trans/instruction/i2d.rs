use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct I2D;

impl Instruction for I2D {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::i2d,
            icp: 0,
        };

        (info, pc + 1)
    }
}
