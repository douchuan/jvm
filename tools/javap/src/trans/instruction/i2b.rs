use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct I2B;

impl Instruction for I2B {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::i2b,
            icp: 0,
        };

        (info, pc + 1)
    }
}
