use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Dastore;

impl Instruction for Dastore {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::dastore.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}