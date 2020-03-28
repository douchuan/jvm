use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Bastore;

impl Instruction for Bastore {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::bastore.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}