use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Ret;

impl Instruction for Ret {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::ret.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 2)
   }
}