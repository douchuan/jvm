use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Iflt;

impl Instruction for Iflt {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::iflt.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 3)
   }
}