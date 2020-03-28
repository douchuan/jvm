use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Astore_0;

impl Instruction for Astore_0 {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::astore_0.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}