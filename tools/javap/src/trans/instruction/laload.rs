use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Laload;

impl Instruction for Laload {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::laload.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}