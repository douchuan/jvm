use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Drem;

impl Instruction for Drem {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::drem.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}