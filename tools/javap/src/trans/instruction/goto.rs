use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Goto;

impl Instruction for Goto {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::goto.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 3)
   }
}