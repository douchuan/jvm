use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Getfield;

impl Instruction for Getfield {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let indexbyte1 = codes[pc + 1] as i16;
       let indexbyte2 = codes[pc + 2] as i16;
       let icp = ((indexbyte1 << 8 |indexbyte2) as i32) as usize;
       let info = InstructionInfo {
           name: OpCode::getfield.into(),
           code: codes[pc],
           icp
       };

       (info, pc + 3)
   }
}