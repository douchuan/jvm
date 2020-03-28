use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Ixor;

impl Instruction for Ixor {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::ixor.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}