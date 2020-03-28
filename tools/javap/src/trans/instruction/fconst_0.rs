use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Fconst_0;

impl Instruction for Fconst_0 {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::fconst_0.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}