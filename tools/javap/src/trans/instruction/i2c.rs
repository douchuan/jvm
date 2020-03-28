use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct I2C;

impl Instruction for I2C {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::i2c.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}