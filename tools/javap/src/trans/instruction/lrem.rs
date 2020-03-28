use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Lrem;

impl Instruction for Lrem {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::lrem.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}