use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Fload_2;

impl Instruction for Fload_2 {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::fload_2.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}