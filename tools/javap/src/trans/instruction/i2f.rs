use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct I2F;

impl Instruction for I2F {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::i2f.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}