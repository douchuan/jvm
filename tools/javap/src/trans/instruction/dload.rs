use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Dload;

impl Instruction for Dload {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::dload.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}