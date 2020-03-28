use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Dup_X1;

impl Instruction for Dup_X1 {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::dup_x1.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}