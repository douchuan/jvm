use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct I2D;

impl Instruction for I2D {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::i2d.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}