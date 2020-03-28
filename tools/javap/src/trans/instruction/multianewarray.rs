use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Multianewarray;

impl Instruction for Multianewarray {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::multianewarray.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 4)
   }
}