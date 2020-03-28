use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct If_Acmpne;

impl Instruction for If_Acmpne {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::if_acmpne.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 3)
   }
}