use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Invokeinterface;

impl Instruction for Invokeinterface {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::invokeinterface.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 5)
   }
}