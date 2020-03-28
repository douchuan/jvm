use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Fstore;

impl Instruction for Fstore {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::fstore.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 2)
   }
}