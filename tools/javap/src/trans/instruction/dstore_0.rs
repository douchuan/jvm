use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Dstore_0;

impl Instruction for Dstore_0 {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::dstore_0.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}