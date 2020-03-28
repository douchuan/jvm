use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Bipush;

impl Instruction for Bipush {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::bipush.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 2)
   }
}