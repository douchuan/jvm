use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Astore_1;

impl Instruction for Astore_1 {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::astore_1.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}