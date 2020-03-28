use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Land;

impl Instruction for Land {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::land.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}