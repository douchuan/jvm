use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Irem;

impl Instruction for Irem {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::irem.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}