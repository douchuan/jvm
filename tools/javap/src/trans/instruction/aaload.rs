use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Aaload;

impl Instruction for Aaload {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::aaload.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}