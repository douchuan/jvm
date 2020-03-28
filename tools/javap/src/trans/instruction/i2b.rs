use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct I2B;

impl Instruction for I2B {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::i2b.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}
