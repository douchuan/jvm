use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct D2I;

impl Instruction for D2I {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::d2i.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}