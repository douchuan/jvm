use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Getfield;

impl Instruction for Getfield {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::getfield.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 3)
   }
}