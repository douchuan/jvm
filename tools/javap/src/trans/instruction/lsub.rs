use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Lsub;

impl Instruction for Lsub {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::lsub.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}