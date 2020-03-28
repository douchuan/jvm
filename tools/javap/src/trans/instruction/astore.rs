use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Astore;

impl Instruction for Astore {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::astore.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 2)
   }
}