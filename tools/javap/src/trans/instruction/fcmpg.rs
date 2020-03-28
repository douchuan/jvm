use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Fcmpg;

impl Instruction for Fcmpg {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::fcmpg.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}