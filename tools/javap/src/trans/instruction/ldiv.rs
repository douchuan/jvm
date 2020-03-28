use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Ldiv;

impl Instruction for Ldiv {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::ldiv.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}