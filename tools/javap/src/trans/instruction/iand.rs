use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Iand;

impl Instruction for Iand {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::iand.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}