use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Frem;

impl Instruction for Frem {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::frem.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}