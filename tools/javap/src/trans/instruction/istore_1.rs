use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Istore_1;

impl Instruction for Istore_1 {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::istore_1.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}