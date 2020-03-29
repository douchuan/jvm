use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct I2L;

impl Instruction for I2L {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::i2l,
            icp: 0,
        };

        (info, pc + 1)
    }
}
