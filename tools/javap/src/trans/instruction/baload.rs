use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Baload;

impl Instruction for Baload {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::baload.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}