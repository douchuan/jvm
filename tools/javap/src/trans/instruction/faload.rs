use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Faload;

impl Instruction for Faload {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::faload.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}