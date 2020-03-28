use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Iconst_2;

impl Instruction for Iconst_2 {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::iconst_2.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}