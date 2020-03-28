use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Ireturn;

impl Instruction for Ireturn {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::ireturn.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}