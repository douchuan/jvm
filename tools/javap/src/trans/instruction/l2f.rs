use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct L2F;

impl Instruction for L2F {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::l2f.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}