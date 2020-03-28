use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Saload;

impl Instruction for Saload {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::saload.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}