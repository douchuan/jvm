use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Freturn;

impl Instruction for Freturn {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::freturn.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}