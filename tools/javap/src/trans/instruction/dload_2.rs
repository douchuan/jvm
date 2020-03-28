use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Dload_2;

impl Instruction for Dload_2 {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::dload_2.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}