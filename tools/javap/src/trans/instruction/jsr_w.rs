use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Jsr_W;

impl Instruction for Jsr_W {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::jsr_w.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 5)
   }
}