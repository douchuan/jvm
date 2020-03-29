use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct I2C;

impl Instruction for I2C {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::i2c,
            icp: 0,
        };

        (info, pc + 1)
    }
}
