use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Ineg;

impl Instruction for Ineg {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::ineg.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}