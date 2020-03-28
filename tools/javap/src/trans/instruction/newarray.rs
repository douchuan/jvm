use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Newarray;

impl Instruction for Newarray {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::newarray.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 2)
   }
}