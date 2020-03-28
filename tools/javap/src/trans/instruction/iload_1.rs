use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Iload_1;

impl Instruction for Iload_1 {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::iload_1.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}