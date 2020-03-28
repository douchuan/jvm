use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct New;

impl Instruction for New {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::new.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 3)
   }
}