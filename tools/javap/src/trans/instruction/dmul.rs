use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Dmul;

impl Instruction for Dmul {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::dmul.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}