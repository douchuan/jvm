use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Putfield;

impl Instruction for Putfield {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::putfield.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 3)
   }
}