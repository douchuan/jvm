use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Lconst_1;

impl Instruction for Lconst_1 {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::lconst_1.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}