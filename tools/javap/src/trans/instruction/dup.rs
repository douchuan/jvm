use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Dup;

impl Instruction for Dup {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::dup.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}