use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Getstatic;

impl Instruction for Getstatic {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::getstatic.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 3)
   }
}