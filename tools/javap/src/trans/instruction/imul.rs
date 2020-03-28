use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Imul;

impl Instruction for Imul {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::imul.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}