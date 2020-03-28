use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct If_Icmpge;

impl Instruction for If_Icmpge {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::if_icmpge.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 3)
   }
}