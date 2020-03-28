use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Ishl;

impl Instruction for Ishl {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::ishl.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}