use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Instanceof;

impl Instruction for Instanceof {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::instanceof.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 3)
   }
}