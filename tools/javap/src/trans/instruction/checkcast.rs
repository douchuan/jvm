use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Checkcast;

impl Instruction for Checkcast {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::checkcast.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 3)
   }
}