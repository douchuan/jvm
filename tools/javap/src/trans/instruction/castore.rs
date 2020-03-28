use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Castore;

impl Instruction for Castore {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::castore.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}