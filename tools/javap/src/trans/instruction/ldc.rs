use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Ldc;

impl Instruction for Ldc {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::ldc.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 2)
   }
}