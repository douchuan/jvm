use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Fneg;

impl Instruction for Fneg {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::fneg.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}