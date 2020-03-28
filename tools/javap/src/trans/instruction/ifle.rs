use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Ifle;

impl Instruction for Ifle {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::ifle.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 3)
   }
}