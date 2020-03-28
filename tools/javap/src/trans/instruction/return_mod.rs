use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Return;

impl Instruction for Return {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::return_void.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}