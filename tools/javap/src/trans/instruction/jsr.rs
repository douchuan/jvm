use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Jsr;

impl Instruction for Jsr {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::jsr.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 3)
   }
}