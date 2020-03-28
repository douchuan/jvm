use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct If_Acmpeq;

impl Instruction for If_Acmpeq {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::if_acmpeq.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 3)
   }
}