use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Invokespecial;

impl Instruction for Invokespecial {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::invokespecial.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 3)
   }
}