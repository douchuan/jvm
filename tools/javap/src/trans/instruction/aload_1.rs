use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Aload_1;

impl Instruction for Aload_1 {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::aload_1.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}