use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Putstatic;

impl Instruction for Putstatic {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::putstatic.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 3)
   }
}