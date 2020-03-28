use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Dup2_X2;

impl Instruction for Dup2_X2 {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::dup2_x2.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}