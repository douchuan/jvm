use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Aastore;

impl Instruction for Aastore {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::aastore.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}