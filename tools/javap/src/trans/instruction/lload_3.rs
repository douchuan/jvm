use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Lload_3;

impl Instruction for Lload_3 {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::lload_3.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}