use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Ifgt;

impl Instruction for Ifgt {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::ifgt.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 3)
   }
}