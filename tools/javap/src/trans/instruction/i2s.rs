use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct I2S;

impl Instruction for I2S {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::i2s,
            icp: 0,
            wide: false,
        };

        (info, pc + 1)
    }
}
