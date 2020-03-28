use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct D2F;

impl Instruction for D2F {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::d2f.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}