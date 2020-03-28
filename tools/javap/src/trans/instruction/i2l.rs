use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct I2L;

impl Instruction for I2L {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::i2l.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}